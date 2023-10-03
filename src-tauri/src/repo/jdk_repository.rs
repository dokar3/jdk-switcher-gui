use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
};

use crate::{errors::AppError, model::jdk::Jdk, util::paths};

pub struct JdkRepository {}

#[allow(dead_code)]
impl JdkRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Get all saved jdk.
    pub fn get_all(&self) -> Result<Vec<Jdk>, AppError> {
        let file = self.json_file(false, false)?;
        Ok(serde_json::from_reader::<File, Vec<Jdk>>(file)?)
    }

    /// Add a jdk.
    pub fn add(&self, jdk: &Jdk) -> Result<(), AppError> {
        let mut all = self.get_all().unwrap_or_default();
        if all.iter().any(|item| item.path == jdk.path) {
            return self.update(jdk);
        }
        all.push(jdk.clone());
        self.save_jdks(&all)
    }

    /// Add a list of jdk.
    pub fn add_all(&self, jdks: &Vec<Jdk>) -> Result<(), AppError> {
        let mut all = self.get_all().unwrap_or_default();
        let mut map = HashMap::<String, usize>::new();
        for (index, jdk) in all.iter().enumerate() {
            map.insert(jdk.path.clone(), index);
        }
        for jdk in jdks {
            if let Some(index) = map.get(&jdk.path) {
                // Already exists, update
                all[*index] = jdk.clone();
            } else {
                // Add to last
                all.push(jdk.clone())
            }
        }
        self.save_jdks(&all)
    }

    /// Update a jdk.
    pub fn update(&self, jdk: &Jdk) -> Result<(), AppError> {
        let mut all = self.get_all().unwrap_or_default();
        let Some(index) = all.iter().position(|item| item.path == jdk.path) else {
            return  Err(AppError::new(format!("Jdk '{}' does not exist.", jdk.path)));
        };
        all[index] = jdk.clone();
        self.save_jdks(&all)
    }

    /// Remove a jdk by its path.
    pub fn remove_by_path(&self, path: &str) -> Result<(), AppError> {
        let mut all = self.get_all().unwrap_or_default();
        let Some(index) = all.iter().position(|item| item.path == path) else {
            return  Err(AppError::new(format!("Jdk '{}' does not exist.", path)));
        };
        all.remove(index);
        self.save_jdks(&all)
    }

    /// Remove a jdk.
    pub fn remove(&self, jdk: &Jdk) -> Result<(), AppError> {
        let mut all = self.get_all().unwrap_or_default();
        let Some(index) = all.iter().position(|item| item.path == jdk.path) else {
            return  Err(AppError::new(format!("Jdk '{}' does not exist.", jdk.path)));
        };
        all.remove(index);
        self.save_jdks(&all)
    }

    /// Clear all jdks (delete the store file).
    pub fn clear(&self) -> Result<(), ()> {
        let file_buf = paths::jdks_json_path();
        if !file_buf.exists() {
            return Ok(());
        }
        fs::remove_file(file_buf).map_err(|_e| ())
    }

    fn save_jdks(&self, jdks: &Vec<Jdk>) -> Result<(), AppError> {
        let mut file = self.json_file(true, true)?;
        Ok(serde_json::to_writer(&mut file, jdks)?)
    }

    fn json_file(
        &self,
        create_if_not_exists: bool,
        clear_content: bool,
    ) -> Result<File, AppError> {
        let file_buf = paths::jdks_json_path();
        if !file_buf.exists() {
            if create_if_not_exists {
                let Some(parent) = file_buf.parent() else {
                    let message = format!("Cannot find parent of '{}'", file_buf.display());
                    return Err(AppError::new(message));
                };
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
                Ok(File::create(file_buf)?)
            } else {
                Err(AppError::new(format!(
                    "File '{}' not exists",
                    file_buf.display()
                )))
            }
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(clear_content)
                .open(file_buf)
                .map_err(|e| e.into())
        }
    }
}
