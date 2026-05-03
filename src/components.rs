use image::DynamicImage;

// コンポーネントを表す構造体
pub struct Vcomponent {
    cached_html: String,
    cached_css: String,
    children: Vec<Vcomponent>,
    content: Vec<VContent>,
}

// TODO: 画像対応
pub struct VContent {
    string: Option<String>,
    image: Option<DynamicImage>,
}

impl Vcomponent {
    pub fn from_str(document: &'static str) -> Self {
        let (html, css) = split_embedded_style(document);

        Self {
            cached_html: html,
            cached_css: css,
            children: Vec::new(),
            content: Vec::new(),
        }
    }

    pub fn child(mut self, component: Vcomponent) -> Self {
        self.children.push(component);
        self
    }

    pub fn children(mut self, components: impl IntoIterator<Item =Vcomponent>) -> Self {
        self.children.extend(components);
        self
    }

    pub fn render(&self) -> String {
        let children_html: String = self
            .children
            .iter()
            .map(|c| c.render())
            .collect::<Vec<_>>()
            .join("\n");

        self.cached_html
            .replace("<Children />", &children_html)
            .replace("<Children/>", &children_html)
            .replace("<Children></Children>", &children_html)
    }

    pub fn css(&self) -> String {
        let mut all_css = vec![self.cached_css.clone()];
        for child in &self.children {
            let child_css = child.css();
            if !child_css.is_empty() {
                all_css.push(child_css);
            }
        }
        merge_css(&all_css.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }
}

impl VContent {
    pub fn string(s: String) -> Self {
        Self {
            string: Option::from(s),
            image: None,
        }
    }

    pub fn image(img: DynamicImage) -> Self {
        Self {
            string: None,
            image: Option::from(img),
        }
    }
}

#[macro_export]
macro_rules! components_list {
    ($($name:ident),* $(,)?) => {
        $(
            fn $name() -> Vcomponent {
                Vcomponent::from_str(include_str!(concat!(
                    "../resources/components/",
                    stringify!($name),
                    ".html"
                )))
            }
        )*
    };
}

fn split_embedded_style(document: &str) -> (String, String) {
    let open_tag = "<style>";
    let close_tag = "</style>";
    if let (Some(open), Some(close)) = (document.find(open_tag), document.find(close_tag)) {
        if close > open {
            let css_start = open + open_tag.len();
            let css = document[css_start..close].trim().to_string();
            let mut html = String::with_capacity(document.len() - (close + close_tag.len() - open));
            html.push_str(document[..open].trim());
            html.push('\n');
            html.push_str(document[close + close_tag.len()..].trim());
            return (html, css);
        }
    }
    (document.to_string(), String::new())
}

fn merge_css(parts: &[&str]) -> String {
    let mut css = String::new();
    for part in parts {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(p);
    }
    css
}