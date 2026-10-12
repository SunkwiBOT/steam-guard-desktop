use guard_core::error_code::ErrorCode;

use super::AppState;

impl AppState {
    pub fn create_folder(
        &self,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err(ErrorCode::FolderNameRequired.to_string());
        }
        self.update_settings(|settings| {
            let id = settings.create_folder(name);
            settings.set_folder_color(&id, color);
            settings.set_folder_icon(&id, icon);
        })
    }

    pub fn update_folder(
        &self,
        id: &str,
        name: &str,
        color: Option<&str>,
        icon: Option<&str>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err(ErrorCode::FolderNameRequired.to_string());
        }
        self.update_known_folder(id, |settings| {
            settings.rename_folder(id, name);
            settings.set_folder_color(id, color);
            settings.set_folder_icon(id, icon);
        })
    }

    pub fn delete_folder(&self, id: &str) -> Result<(), String> {
        self.update_settings(|settings| {
            settings.delete_folder(id);
        })
    }

    pub fn set_account_folder(&self, steam_id: &str, folder: Option<&str>) -> Result<(), String> {
        self.update_settings(|settings| settings.set_account_folder(steam_id, folder))
    }

    pub fn set_account_color(&self, steam_id: &str, color: Option<&str>) -> Result<(), String> {
        self.update_settings(|settings| settings.set_account_color(steam_id, color))
    }

    pub fn set_folder_color(&self, id: &str, color: Option<&str>) -> Result<(), String> {
        self.update_known_folder(id, |settings| {
            settings.set_folder_color(id, color);
        })
    }

    pub fn set_folder_icon(&self, id: &str, icon: Option<&str>) -> Result<(), String> {
        self.update_known_folder(id, |settings| {
            settings.set_folder_icon(id, icon);
        })
    }

    /// Folder and colour of an account that is no longer listed. Presentation
    /// only, so a failed write is logged rather than reported.
    pub fn forget_account(&self, steam_id: &str) {
        if let Err(error) = self.update_settings(|settings| settings.forget_account(steam_id)) {
            crate::logging::write(&format!(
                "action=forget_account_settings status=failed error={}",
                crate::logging::error_category(&error)
            ));
        }
    }

    /// The existence check and the change run under the same lock, so a
    /// concurrent delete cannot slip in between.
    fn update_known_folder(
        &self,
        id: &str,
        change: impl FnOnce(&mut crate::settings::Settings),
    ) -> Result<(), String> {
        let mut inner = self.lock();
        if !inner
            .settings
            .data()
            .folders
            .iter()
            .any(|folder| folder.id == id)
        {
            return Err(ErrorCode::UnknownFolder.to_string());
        }
        inner.settings.update(change)
    }
}
