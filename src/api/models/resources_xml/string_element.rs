use strong_xml::{XmlRead, XmlWrite};

#[derive(XmlWrite, XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "string")]
pub struct StringElement {
    #[xml(child = "name")]
    pub name: String,
    #[xml(text)]
    pub content: String,
}

/*
assert_eq!(
    (StringElement {
        name: "test_name",
        content: "test_content" }
    ).to_string().unwrap(),
    r#"<string name="test_name">test_content</string>"#
);

assert_eq!(
    StringElement::from_str(r#"<string name="test_name">test_content</string>"#).unwrap(),
    StringElement {
        name: "test_name",
        content: "test_content"
    }
);
*/
