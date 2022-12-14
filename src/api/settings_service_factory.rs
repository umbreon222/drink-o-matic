use std::fs;
use std::sync::RwLock;
use rocket::serde::json::serde_json;
use crate::api::models::settings::Settings;
use crate::SettingsService;

pub struct SettingsServiceFactory {}

impl SettingsServiceFactory {
    pub fn create_or_panic(number_of_pumps: u8) -> SettingsService {
        let home_dir = dirs::home_dir().unwrap();
        let settings_file_path = dotenv::var("SETTINGS_FILE_PATH").unwrap();
        let file_path = home_dir.join(settings_file_path);
        let settings: Settings;
        match fs::read_to_string(file_path.clone()) {
            Ok(existing_settings_json) => {
                settings = serde_json::from_str(&existing_settings_json).unwrap();
            }
            Err(_) => {
                settings = Settings::new(number_of_pumps);
            }
        }
        
        SettingsService {
            settings: RwLock::new(settings),
            settings_file_path: file_path.into_boxed_path()
        }
    }
}