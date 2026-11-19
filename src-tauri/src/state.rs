use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use guard_core::authenticator::Authenticator;
use guard_core::profile::ProfileStore;
use guard_core::storage::Database;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::settings::{Settings, SettingsStore};
use guard_core::error_code::ErrorCode;
mod folders;

pub struct Inner {
    pub settings: SettingsStore,
    pub unlocked_at: Option<Instant>,
    pub login_names: BTreeMap<String, String>,
    pub account_names: BTreeMap<String, String>,
    // ! Paths chosen through the native file picker stay here: the webview only receives the display name.
    pub picked_mafile: Option<PathBuf>,
    pub picked_backup: Option<PathBuf>,
}

pub struct AppState {
    inner: Mutex<Inner>,
    // ! Own mutex so a long network call never blocks the settings, the tray or the window close handler.
    authenticator: Arc<Mutex<Authenticator>>,
    pub profiles: ProfileStore,
    snapshot_revision: AtomicU64,
    tray_available: AtomicBool,
    // ! The app must not exit while this is non-zero, or a link Steam already accepted could be lost mid-flight.
    link_operations: AtomicUsize,
    // ! `restart()` cannot be vetoed once called: a link started after an update install began
    // could be cut off mid-flight with no chance to finish or roll back.
    update_installing: AtomicBool,
    corrupt_db_backup: Mutex<Option<String>>,
    // ! The clock-offset loop starts only after the page registered its `clock-drift-warning` listener.
    frontend_ready: AtomicBool,
}

pub struct LinkOperationGuard<'a> {
    counter: &'a AtomicUsize,
}

impl Drop for LinkOperationGuard<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (database, corrupt_db_backup) = open_database();
        Self {
            inner: Mutex::new(Inner {
                settings: SettingsStore::load(database.clone()),
                unlocked_at: None,
                login_names: BTreeMap::new(),
                account_names: BTreeMap::new(),
                picked_mafile: None,
                picked_backup: None,
            }),
            authenticator: Arc::new(Mutex::new(Authenticator::new(database.clone()))),
            profiles: ProfileStore::new(database),
            snapshot_revision: AtomicU64::new(0),

            tray_available: AtomicBool::new(false),
            link_operations: AtomicUsize::new(0),
            update_installing: AtomicBool::new(false),
            corrupt_db_backup: Mutex::new(corrupt_db_backup),
            frontend_ready: AtomicBool::new(false),
        }
    }

    pub fn take_corrupt_db_backup(&self) -> Option<String> {
        self.corrupt_db_backup
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }

    pub fn mark_frontend_ready(&self) -> bool {
        !self.frontend_ready.swap(true, Ordering::SeqCst)
    }

    // ! Set once the downloaded update starts replacing the running files, right before the
    // restart it leads to; never cleared, since the only way out of this state is that restart.
    pub fn begin_update_install(&self) {
        self.update_installing.store(true, Ordering::SeqCst);
    }

    pub fn begin_link_operation(&self) -> Result<LinkOperationGuard<'_>, String> {
        if self.update_installing.load(Ordering::SeqCst) {
            return Err(ErrorCode::AuthenticatorBusy.to_string());
        }
        self.link_operations.fetch_add(1, Ordering::SeqCst);
        Ok(LinkOperationGuard {
            counter: &self.link_operations,
        })
    }

    pub fn link_operation_in_flight(&self) -> bool {
        self.link_operations.load(Ordering::SeqCst) > 0
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn authenticator(&self) -> Arc<Mutex<Authenticator>> {
        Arc::clone(&self.authenticator)
    }

    pub fn update_settings(&self, change: impl FnOnce(&mut Settings)) -> Result<(), String> {
        self.lock().settings.update(change)
    }

    pub fn settings(&self) -> Settings {
        self.lock().settings.data().clone()
    }

    pub fn next_snapshot_revision(&self) -> u64 {
        self.snapshot_revision.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_picked_mafile(&self, path: PathBuf) {
        self.lock().picked_mafile = Some(path);
    }

    // ! Read, not taken: a failed import must leave the path in place for the retry.
    pub fn picked_mafile(&self) -> Option<PathBuf> {
        self.lock().picked_mafile.clone()
    }

    pub fn clear_picked_mafile(&self) {
        self.lock().picked_mafile = None;
    }

    pub fn set_picked_backup(&self, path: PathBuf) {
        self.lock().picked_backup = Some(path);
    }

    pub fn picked_backup(&self) -> Option<PathBuf> {
        self.lock().picked_backup.clone()
    }

    pub fn clear_picked_backup(&self) {
        self.lock().picked_backup = None;
    }

    pub fn set_tray_available(&self, available: bool) {
        self.tray_available.store(available, Ordering::Relaxed);
    }

    pub fn tray_available(&self) -> bool {
        self.tray_available.load(Ordering::Relaxed)
    }

    pub fn note_vault_unlocked(&self) {
        self.lock().unlocked_at = Some(Instant::now());
    }

    pub fn note_vault_locked(&self) {
        self.lock().unlocked_at = None;
    }

    pub fn lock_vault_if_due(&self) -> bool {
        let elapsed = {
            let inner = self.lock();
            let Some(unlocked_at) = inner.unlocked_at else {
                return false;
            };
            let elapsed = unlocked_at.elapsed().as_secs();
            if !inner
                .settings
                .data()
                .vault_lock_interval
                .locks_after(elapsed)
            {
                return false;
            }
            elapsed
        };
        let locked = lock_authenticator(&self.authenticator).lock().is_ok();
        if !locked {
            return false;
        }
        crate::logging::write(&format!(
            "action=lock_vault status=locked reason=idle duration={elapsed}s"
        ));
        self.lock().unlocked_at = None;
        true
    }

    pub fn remember_login_name(&self, session_id: &str, account_name: &str) {
        self.lock()
            .login_names
            .insert(session_id.to_string(), account_name.to_string());
    }

    pub fn take_login_name(&self, session_id: &str) -> Option<String> {
        self.lock().login_names.remove(session_id)
    }

    pub fn remember_account_name(&self, steam_id: &str, account_name: &str) {
        if !account_name.is_empty() {
            self.lock()
                .account_names
                .insert(steam_id.to_string(), account_name.to_string());
        }
    }

    pub fn account_name(&self, steam_id: &str) -> Option<String> {
        self.lock().account_names.get(steam_id).cloned()
    }
}

pub fn lock_authenticator(authenticator: &Mutex<Authenticator>) -> MutexGuard<'_, Authenticator> {
    authenticator
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub fn exit_blocked(app: &AppHandle) -> bool {
    let in_flight = app.state::<AppState>().link_operation_in_flight();
    if in_flight {
        crate::logging::write("action=request_exit status=blocked reason=link_in_flight");
        crate::show_main_window(app);
        let locale = app.state::<AppState>().settings().language.resolved();
        let message = rust_i18n::t!("exit_blocked_link_in_flight", locale = locale).to_string();
        // ! Non-blocking: this runs inside tao's event loop, which owns the GLib idle dispatch a blocking rfd dialog needs.
        app.dialog()
            .message(message)
            .title(crate::APP_NAME)
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    }
    in_flight
}

pub fn request_exit(app: &AppHandle) {
    if !exit_blocked(app) {
        app.exit(0);
    }
}

pub fn request_close(app: &AppHandle, api: &tauri::CloseRequestApi) {
    if exit_blocked(app) {
        api.prevent_close();
    } else {
        app.exit(0);
    }
}

// ! Kept here so the `t!` macro stays declared exactly once, in `lib.rs`.
pub fn database_recovered_message(locale: &str, backup: &str) -> String {
    rust_i18n::t!(
        "database_recovered_notice",
        locale = locale,
        backup = backup
    )
    .to_string()
}

// ! Startup stops without the database: running on would lose a newly saved Steam Guard secret on restart.
fn open_database() -> (Database, Option<String>) {
    let locale = crate::locale::Language::Auto.resolved();
    let opened = match Database::open(&crate::settings::database_path()) {
        Ok(opened) => opened,
        Err(error) => {
            // ! No window or panic hook yet and the language preference lives in the unopened database, so the system locale is used.
            crate::logging::write(&format!(
                "action=open_database status=failed code={} context={}",
                error.code(),
                error.context().unwrap_or("")
            ));
            let path = crate::settings::database_path();
            let message = rust_i18n::t!(
                "database_open_failed_body",
                locale = locale,
                code = error.code().to_string(),
                path = path.display().to_string()
            )
            .to_string();
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title(crate::APP_NAME)
                .set_description(message)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            std::process::exit(1);
        }
    };
    let backup_name = opened.moved_aside.as_ref().map(|backup| {
        let name = backup
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        crate::logging::write(&format!(
            "action=open_database status=recovered category=database_corrupt backup={name}"
        ));
        name
    });
    (opened.database, backup_name)
}
