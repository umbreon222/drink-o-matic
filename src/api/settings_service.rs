use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::sync::{ Arc, RwLock };
use crate::api::models::settings::Settings;
use crate::api::ResourceService;

pub struct SettingsService {
    resource_service: Arc<ResourceService>,
    pub settings: RwLock<Settings>,
    settings_file_path: PathBuf
}

impl SettingsService {
    pub fn new(resource_service: Arc<ResourceService>, settings: RwLock<Settings>, settings_file_path: PathBuf) -> SettingsService {
        SettingsService { resource_service, settings, settings_file_path }
    }

    pub fn save(&self, settings: Settings) -> Result<(), String> {
        match serde_json::to_string(&settings) {
            Ok(settings_json) => {
                match fs::create_dir_all(self.settings_file_path.parent().unwrap()) {
                    Ok(_) => {
                        match OpenOptions::new().write(true).create(true).open(self.settings_file_path.clone()) {
                            Ok(mut settings_file) => {
                                if settings_file.set_len(0).is_err() {
                                    return Err(self.resource_service.get_resource_string_by_name("truncating_settings_file_error_message").unwrap());
                                }
                                match settings_file.write_all(settings_json.as_bytes()) {
                                    Ok(_) => {
                                        *self.settings.write().unwrap() = settings;
                                        Ok(())
                                    },
                                    Err(error) => Err(self.resource_service.get_resource_string_by_name("write_to_settings_file_error_message_template").unwrap() + &error.to_string())
                                }
                            }
                            Err(error) => Err(self.resource_service.get_resource_string_by_name("create_or_open_settings_file_error_message_template").unwrap() + &error.to_string())
                        }
                    },
                    Err(error) => {
                        return Err(self.resource_service.get_resource_string_by_name("create_settings_directory_error_message_template").unwrap() + &error.to_string())
                    }
                }
            }
            Err(error) => Err(self.resource_service.get_resource_string_by_name("settings_serialization_error_message_template").unwrap() + &error.to_string())
        }
    }
}
