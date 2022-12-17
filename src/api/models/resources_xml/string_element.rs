use hard_xml::{XmlRead, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "string")]
pub struct StringElement {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(text)]
    pub content: String,
}
