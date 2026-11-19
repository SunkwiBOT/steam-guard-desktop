use std::path::{Path, PathBuf};

use guard_core::authenticator::{
    CodeInfo, ConfirmationItem, ConfirmationKind, GuardKind, GuardMethod, LinkConfirmKind,
    LinkStart, LinkStatus, LoginBegin, LoginResult, StoredAuthenticator,
};
use guard_core::error_code::ErrorCode;
use steam_guard_desktop_lib::auth_commands::backup::BackupImportDto;
use steam_guard_desktop_lib::auth_commands::files::PickKind;
use steam_guard_desktop_lib::commands::dto::{
    AccountDto, AppInfoDto, FolderDto, SettingsDto, SnapshotDto, VaultStatusDto,
};
use steam_guard_desktop_lib::commands::updates::{UpdateInfoDto, UpdateProgressDto};
use steam_guard_desktop_lib::locale::Language;
use steam_guard_desktop_lib::logging::LogSnapshot;
use steam_guard_desktop_lib::open_url::UrlDestination;
use steam_guard_desktop_lib::settings::{Folder, SearchScope, UpdateInterval, VaultLockInterval};
use ts_rs::{Config, TS};

fn config(out_dir: &Path) -> Config {
    Config::new().with_out_dir(out_dir)
}

fn export<T: TS + 'static>(cfg: &Config) {
    T::export_all(cfg).expect("bindings written");
}

fn main() {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: export-bindings <output directory>"),
    );

    if dir.exists() {
        for entry in std::fs::read_dir(&dir).expect("readable output directory") {
            let path = entry.expect("directory entry").path();

            if path.extension().is_some_and(|e| e == "ts") {
                std::fs::remove_file(path).expect("stale binding removed");
            }
        }
    }

    let cfg = config(&dir);

    export::<SnapshotDto>(&cfg);
    export::<AccountDto>(&cfg);
    export::<VaultStatusDto>(&cfg);
    export::<FolderDto>(&cfg);
    export::<SettingsDto>(&cfg);
    export::<AppInfoDto>(&cfg);
    export::<Folder>(&cfg);
    export::<Language>(&cfg);
    export::<UpdateInterval>(&cfg);
    export::<SearchScope>(&cfg);
    export::<VaultLockInterval>(&cfg);
    export::<StoredAuthenticator>(&cfg);
    export::<CodeInfo>(&cfg);
    export::<ConfirmationKind>(&cfg);
    export::<ConfirmationItem>(&cfg);
    export::<GuardKind>(&cfg);
    export::<GuardMethod>(&cfg);
    export::<LoginBegin>(&cfg);
    export::<LoginResult>(&cfg);
    export::<LinkStatus>(&cfg);
    export::<LinkConfirmKind>(&cfg);
    export::<LinkStart>(&cfg);
    export::<LogSnapshot>(&cfg);
    export::<BackupImportDto>(&cfg);
    export::<UpdateInfoDto>(&cfg);
    export::<UpdateProgressDto>(&cfg);
    export::<ErrorCode>(&cfg);
    export::<PickKind>(&cfg);
    export::<UrlDestination>(&cfg);
}
