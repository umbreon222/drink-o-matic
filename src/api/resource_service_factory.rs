use std::fs;
use hard_xml::XmlRead;
use crate::api::models::resources_xml::ResourcesElement;
use crate::api::ResourceService;

pub struct ResourceServiceFactory {}

impl ResourceServiceFactory {
    pub fn create_or_panic() -> ResourceService {
        let home_dir = dirs::home_dir().unwrap();
        let strings_xml_file_path = dotenv::var("STRINGS_XML_FILE_PATH").unwrap();
        let file_path = home_dir.join(strings_xml_file_path);
        let resource_xml_content = fs::read_to_string(file_path).unwrap();
        let resource_element = ResourcesElement::from_str(&resource_xml_content).unwrap();
        ResourceService::new(resource_element)
    }
}
