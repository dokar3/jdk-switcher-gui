use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_channel::{Receiver, Sender};
use tokio::runtime::Runtime;

use crate::{
    app_settings::AppSettings,
    jdk_finder::{find_jdk_from_exe_path, find_jdks_from_dir},
    jdk_switcher,
    model::{jdk::Jdk, settings::SettingsValues},
    repo::jdk_repository::JdkRepository,
    util::{self, paths::find_command_exe_path}, errors::AppError,
};

#[derive(Clone, serde::Serialize)]
pub struct AppUiState {
    pub settings: SettingsValues,
    pub jdks: Vec<Jdk>,
}

pub struct AppViewModel {
    jdk_repo: JdkRepository,
    tokio_runtime: Runtime,
    ui_state: Mutex<AppUiState>,
    state_sender: Arc<Sender<AppUiState>>,
    state_receiver: Receiver<AppUiState>,
}

#[allow(dead_code)]
impl AppViewModel {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded::<AppUiState>();
        Self {
            jdk_repo: JdkRepository::new(),
            tokio_runtime: Runtime::new().unwrap(),
            ui_state: Mutex::new(AppUiState {
                settings: AppSettings::load(),
                jdks: vec![],
            }),
            state_sender: Arc::new(sender),
            state_receiver: receiver,
        }
    }

    /// Load all saved jdks.
    pub fn load_jdks(&self) {
        let jdks = self.jdk_repo.get_all().unwrap_or_default();
        let jdks = self.process_saved_jdks(jdks);
        self.update_ui_state(|state| AppUiState {
            settings: state.settings.clone(),
            jdks,
        });
    }

    pub fn remove_jdk_by_path(&self, path: &str) -> Result<(), AppError> {
        let ret = self.jdk_repo.remove_by_path(path);
        if ret.is_ok() {
            self.load_jdks();
        }
        ret
    }

    /// Scan jdk recursively from a path.
    pub fn try_add_jdks_from_dir(&self, path: &str) -> Result<usize, AppError> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(AppError::new("Dir does not exist."));
        }
        let jdks = find_jdks_from_dir(&path)?;
        let ret = self.jdk_repo.add_all(&jdks).map(|_| jdks.len());
        if ret.is_ok() {
            self.load_jdks();
        }
        ret
    }

    pub fn add_jdk(&self, jdk: &Jdk) {
        self.jdk_repo
            .add(jdk)
            .expect(format!("Cannot add jdk '{}'", jdk.name).as_str());
        self.load_jdks();
    }

    pub fn switch_to_jdk(&self, jdk: &Jdk) -> Result<(), AppError> {
        let ret = jdk_switcher::switch_to_jdk(jdk);
        if ret.is_ok() {
            self.load_jdks()
        }
        ret
    }

    pub fn update_app_theme(&self, theme: &str) -> Result<(), AppError> {
        let mut settings = self.ui_state.lock().unwrap().settings.clone();
        settings.theme = theme.to_string();
        AppSettings::update(&settings)?;
        self.update_ui_state(|state| AppUiState {
            settings,
            jdks: state.jdks.clone(),
        });
        Ok(())
    }

    pub fn update_skip_dir_selection_hint(
        &self,
        value: bool,
    ) -> Result<(), AppError> {
        let mut settings = self.ui_state.lock().unwrap().settings.clone();
        settings.skip_dir_selection_hint = value;
        AppSettings::update(&settings)?;
        self.update_ui_state(|state| AppUiState {
            settings,
            jdks: state.jdks.clone(),
        });
        Ok(())
    }

    // Get the ui state stream to receive incoming updates.
    pub fn ui_state_stream(&self) -> &Receiver<AppUiState> {
        self.notify_ui_state();
        &self.state_receiver
    }

    // Update the current ui state.
    fn update_ui_state<F>(&self, closure: F)
    where
        F: FnOnce(&AppUiState) -> AppUiState,
    {
        let curr = self.ui_state.lock().unwrap().clone();
        let updated_state = closure(&curr);
        *self.ui_state.lock().unwrap() = updated_state.clone();
        self.notify_ui_state();
    }

    // Send the latest ui state to the channel.
    fn notify_ui_state(&self) {
        let state = self.ui_state.lock().unwrap().clone();
        let state_sender = self.state_sender.clone();
        self.tokio_runtime
            .spawn(async move { state_sender.send(state).await.unwrap() });
    }

    /// Function to validate jdk path, check for the current jdk, etc.
    ///
    /// This function will always add/update the current jdk to the list if a jdk has added
    /// to the PATH.
    fn process_saved_jdks(&self, jdks: Vec<Jdk>) -> Vec<Jdk> {
        if let Err(e) = util::env::use_sys_env_path_var() {
            eprintln!(
                "Failed to refresh env before getting the java exe path: {}",
                e
            );
        }

        let Some(java_path) = find_command_exe_path("java") else {
            // Jdk not added to PATH
            return AppViewModel::validate_jdks(jdks);
        };
        let Some(java_bin_dir) = java_path.parent() else {
            // Maybe will never happen
            return AppViewModel::validate_jdks(jdks);
        };

        let Some(current_index) = jdks.iter().position(|item| {
            Path::new(&item.path) == java_bin_dir
        }) else {
            // The current jdk is not in the saved list.
            let Ok(mut current) = find_jdk_from_exe_path(&java_path) else {
                return AppViewModel::validate_jdks(jdks);
            };
            current.is_current = true;
            let mut new_list = jdks.clone();
            // Add current jdk to the list
            new_list.push(current);
            return AppViewModel::validate_jdks(new_list);
        };

        // Update the current jdk
        let mut current = jdks[current_index].clone();
        current.is_current = true;
        let mut new_list = jdks;
        new_list[current_index] = current;

        AppViewModel::validate_jdks(new_list)
    }

    fn validate_jdks(list: Vec<Jdk>) -> Vec<Jdk> {
        list.iter()
            .map(|item| {
                let mut item = item.clone();
                item.is_valid = Path::new(&item.path).exists();
                item
            })
            .collect()
    }
}
