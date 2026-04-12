#[derive(Debug)]
pub struct CssDecl {
    pub selector: &'static str,
    pub property: &'static str,
    pub value: &'static str,
}

#[derive(Debug)]
pub struct ComponentTemplate {
    pub name: &'static str,
    pub root_tag: &'static str,
    pub class_name: &'static str,
    pub hx_get: Option<&'static str>,
    pub hx_post: Option<&'static str>,
    pub declarations: &'static [CssDecl],
}

include!(concat!(env!("OUT_DIR"), "/components_generated.rs"));

pub fn templates() -> &'static [ComponentTemplate] {
    COMPONENT_TEMPLATES
}
