use std::collections::BTreeMap;

use super::dom::{DomDocument, DomNode, DomNodeKind};
use super::parser::Stylesheet;

pub type StyleMap = BTreeMap<String, String>;

#[derive(Debug, Clone)]
pub struct StyledTree {
    pub root: StyledNode,
}

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node: DomNode,
    pub styles: StyleMap,
    pub children: Vec<StyledNode>,
}

pub fn compute_styles(dom: &DomDocument, stylesheet: &Stylesheet) -> StyledTree {
    StyledTree {
        root: style_node(&dom.root, stylesheet),
    }
}

fn style_node(node: &DomNode, stylesheet: &Stylesheet) -> StyledNode {
    let mut styles = BTreeMap::new();

    if let DomNodeKind::Element(element) = &node.kind {
        for rule in &stylesheet.rules {
            if selector_matches(&rule.selector, &element.tag_name) {
                styles.extend(rule.declarations.clone());
            }
        }
    }

    let children = node
        .children
        .iter()
        .map(|child| style_node(child, stylesheet))
        .collect();

    StyledNode {
        node: node.clone(),
        styles,
        children,
    }
}

fn selector_matches(selector: &str, tag_name: &str) -> bool {
    selector == "*" || selector == tag_name
}
