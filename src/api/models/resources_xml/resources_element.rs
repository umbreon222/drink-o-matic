use hard_xml::{XmlRead, XmlWrite};
use crate::api::models::resources_xml::StringElement;

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "resources")]
pub struct ResourcesElement {
    #[xml(child = "string")]
    pub strings: Vec<StringElement>,
}
