use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use guard_core::authenticator::Authenticator;
use guard_core::profile::ProfileStore;
use guard_core::storage::Database;

use crate::settings::{Settings, SettingsStore};

mod folders;

pub struct Inner {
    pub settings: SettingsStore,
    pub unlocked_at: Option<Instant>,
    /// Login names given to password sign-ins, keyed by session until the
    /// sign-in finishes, then by Steam ID: a brand-new account has no vault
    /// entry to take its name from when the link starts.
    pub login_names: BTreeMap<String, String>,
    pub account_names: BTreeMap<String, String>,
}

pub struct AppState {
    inner: Mutex<Inner>,
    /// ! Steam Guard secrets and the login/link sessions, in their own mutex so
    /// a long network call never blocks the settings, the tray or the window
    /// close handler.
    authenticator: Arc<Mutex<Authenticator>>,
    pub profiles: ProfileStore,
    snapshot_revision: AtomicU64,
    window_active: AtomicBool,
    tray_available: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        let database = open_database();
        Self {
            inner: Mutex::new(Inner {
                settings: SettingsStore::load(database.clone()),
                unlocked_at: None,
                login_names: BTreeMap::new(),
                account_names: BTreeMap::new(),
            }),
            authenticator: Arc::new(Mutex::new(Authenticator::new(database.clone()))),
            profiles: ProfileStore::new(database),
            snapshot_revision: AtomicU64::new(0),
            window_active: AtomicBool::new(true),
            tray_available: AtomicBool::new(false),
        }
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

    /// Called while the state is locked so older responses can be discarded.
    pub fn next_snapshot_revision(&self) -> u64 {
        self.snapshot_revision.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_window_active(&self, active: bool) {
        self.window_active.store(active, Ordering::Relaxed);
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

    /// Returns true only when the vault changed from unlocked to locked.
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

/// ! Startup stops without the database: running on would lose a newly saved
/// Steam Guard secret on restart, and the file must stay untouched so a build
/// with the matching schema can still open it.
fn open_database() -> Database {
    let opened = match Database::open(&crate::settings::database_path()) {
        Ok(opened) => opened,
        Err(error) => {
            // No window or panic hook yet: log the reason, then stop.
            crate::logging::write(&format!("action=open_database status=failed code={error}"));
            std::process::exit(1);
        }
    };
    if let Some(backup) = &opened.moved_aside {
        let name = backup
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        crate::logging::write(&format!(
            "action=open_database status=recovered category=database_corrupt backup={name}"
        ));
    }
    opened.database
}
