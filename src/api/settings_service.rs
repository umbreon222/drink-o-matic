use std::path::Path;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::sync::RwLock;
use rocket::serde::json::serde_json;
use crate::api::models::settings::Settings;

const SETTINGS_FILE_PATH: &str = ".drink-o-matic/settings.json";

pub struct SettingsService {
    pub settings: RwLock<Settings>,
    settings_file_path: Box<Path>,
}

impl SettingsService {
    pub fn new() -> Result<Self, String> {
        let home_dir = dirs::home_dir().unwrap();
        let file_path = home_dir.join(SETTINGS_FILE_PATH);
        let settings: Settings;
        match fs::read_to_string(file_path.clone()) {
            Ok(existing_settings_json) => {
                match serde_json::from_str(&existing_settings_json) {
                    Ok(existing_settings) => settings = existing_settings,
                    Err(error) => {
                        return Err(format!("Couldn't parse existing settings: {}", error));
                    }
                }
            }
            Err(_) => {
                settings = Settings::new();
            }
        }
        Ok(SettingsService { settings: RwLock::new(settings), settings_file_path: file_path.into_boxed_path() })
    }

    pub fn save(&self, settings: Settings) -> Result<(), String> {
        match serde_json::to_string(&settings) {
            Ok(settings_json) => {
                match fs::create_dir_all(self.settings_file_path.parent().unwrap()) {
                    Ok(_) => {
                        match OpenOptions::new().write(true).create(true).open(self.settings_file_path.clone()) {
                            Ok(mut settings_file) => {
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