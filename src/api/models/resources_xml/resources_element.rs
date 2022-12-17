use strong_xml::{XmlRead, XmlWrite};
use crate::api::models::resources_xml::StringElement;

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "resources")]
pub struct ResourcesElement {
    #[xml(child = "string")]
    pub strings: Vec<StringElement>,
}

/*
assert_eq!(
    (ResourcesElement {
        strings: vec![
            StringElement {
                name: "test_name1",
                content: "test_content1"
            },
            StringElement {
                name: "test_name2",
                content: "test_content2"
            }
        ]
    }).to_string().unwrap(),
    r#"<resources><string name="test_name1">test_content1</string><string name="test_name2">test_content2</string></resources>"#
);

assert_eq!(
    ResourcesElement::from_str(r#"<resources><string name="test_name1">test_content1</string><string name="test_name2">test_content2</string></resources>"#).unwrap(),
    ResourcesElement {
        strings: vec![
            StringElement {
                name: "test_name1",
                content: "test_content1"
            },
            StringElement {
                name: "test_name2",
                content: "test_content2"
            }
        ]
    }
);
*/