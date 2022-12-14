use std::path::Path;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::sync::RwLock;
use rocket::serde::json::serde_json;
use crate::api::models::settings::Settings;

pub struct SettingsService {
    pub settings: RwLock<Settings>,
    settings_file_path: Box<Path>
}

impl SettingsService {
    pub fn new(settings: RwLock<Settings>, settings_file_path: Box<Path>) -> SettingsService {
        SettingsService { settings, settings_file_path }
    }

    pub fn save(&self, settings: Settings) -> Result<(), String> {
        match serde_json::to_string(&settings) {
            Ok(settings_json) => {
                match fs::create_dir_all(self.settings_file_path.parent().unwrap()) {
                    Ok(_) => {
                        match OpenOptions::new().write(true).create(true).open(self.settings_file_path.clone()) {
                            Ok(mut settings_file) => {
                                if settings_file.set_len(0).is_err() {
                                    return Err(format!("Couldn't truncate settings file"));
                                }
                                match settings_file.write_all(settings_json.as_bytes()) {
                                    Ok(_) => {
                                        *self.settings.write().unwrap() = settings;
                                        Ok(())
                                    },
                                    Err(error) => Err(format!("Couldn't write to settings file: {}", error))
                                }
                            }
                            Err(error) => Err(format!("Couldn't create/open settings file: {}", error))
                        }
                    },
                    Err(error) => {
                        return Err(format!("Couldn't create settings directory: {}", error));
                    }
                }
            }
            Err(error) => Err(format!("Couldn't serialize settings: {}", error))
        }
    }
}
