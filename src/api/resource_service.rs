use handlebars::Handlebars;
use serde::Serialize;
use crate::api::models::resources_xml::ResourcesElement;

#[derive(Clone)]
pub struct ResourceService {
    resource_element: ResourcesElement
}

impl ResourceService{
    pub fn new(resource_element: ResourcesElement) -> ResourceService {
        ResourceService {
            resource_element
        }
    }

    pub fn get_resource_string_by_name(&self, name: &str) -> Option<String> {
        for string_element in &self.resource_element.strings {
            if string_element.name == name {
                return Some(string_element.content.clone())
            }
        }
        None
    }

    pub fn render_resource_template_string_by_name<T: Serialize>(&self, name: &str, data: &T) -> Option<String> {
        let handlebars = Handlebars::new();
        let template_string = self.get_resource_string_by_name(name)?;
        match handlebars.render_template(template_string.as_str(), data) {
            Ok(rendered) => Some(rendered),
            Err(_) => None
        }
    }
}