use std::collections::HashMap;

// イベントハンドラーの型
pub type EventHandler = Box<dyn Fn() + Send + Sync>;

// コンポーネントを表す構造体
pub struct VComponent {
    cached_html: String,
    cached_css: String,
    children: Vec<VComponent>,
    content: Vec<VContent>,
    attributes: HashMap<String, String>,
    handlers: HashMap<String, EventHandler>,
    visible: bool,
}

// TODO: 画像対応
#[allow(unused)]
pub struct VContent {
    string: Option<String>,
    image_path: Option<String>,
}

impl VComponent {
    pub fn from_str(document: &'static str) -> Self {
        let (html, css) = split_embedded_style(document);

        Self {
            cached_html: html,
            cached_css: css,
            children: Vec::new(),
            content: Vec::new(),
            attributes: HashMap::new(),
            handlers: HashMap::new(),
            visible: true,
        }
    }

    // ビルダーメソッド群
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.attributes.insert("label".to_string(), text.into());
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.attributes.insert("id".to_string(), id.into());
        self
    }

    pub fn class(mut self, cls: impl Into<String>) -> Self {
        self.attributes.insert("class".to_string(), cls.into());
        self
    }

    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.content.push(VContent::string(content.into()));
        self
    }

    pub fn image(mut self, path: impl Into<String>) -> Self {
        self.content.push(VContent::image(path.into()));
        self
    }

    pub fn on_click(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.handlers.insert("click".to_string(), Box::new(handler));
        self
    }

    pub fn if_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn new(self) -> Self {
        self
    }

    pub fn child(mut self, component: VComponent) -> Self {
        self.children.push(component);
        self
    }

    pub fn children(mut self, components: impl IntoIterator<Item =VComponent>) -> Self {
        self.children.extend(components);
        self
    }

    pub fn render(&self) -> String {
        if !self.visible {
            return String::new();
        }

        let children_html: String = self
            .children
            .iter()
            .map(|c| c.render())
            .collect::<Vec<_>>()
            .join("\n");

        let mut html = self.cached_html
            .replace("<Children />", &children_html)
            .replace("<Children/>", &children_html)
            .replace("<Children></Children>", &children_html);

        // Content placeholders (e.g. <Content type="Image" />) are expanded here.
        // This keeps the HTML parser/pipeline generic, while allowing VComponent builder APIs.
        for item in &self.content {
            if let Some(path) = &item.image_path {
                // Note: ViewKit's CSS selector support is intentionally minimal; avoid relying
                // on descendant selectors like `.appicon img`.
                let img = format!(
                    "<img class=\"vk-img\" src=\"{}\" />",
                    escape_attr_value(path)
                );
                html = replace_first_content_image(&html, &img);
            } else if let Some(text) = &item.string {
                let escaped = escape_text(text);
                html = replace_first_content_text(&html, &escaped);
            }
        }

        // 属性をHTMLに埋め込む
        for (key, value) in &self.attributes {
            html = html.replace(
                &format!("{{{{ {} }}}}", key),
                value
            );
        }

        html
    }

    pub fn css(&self) -> String {
        let mut all_css = vec![base_css().to_string(), self.cached_css.clone()];
        for child in &self.children {
            let child_css = child.css();
            if !child_css.is_empty() {
                all_css.push(child_css);
            }
        }
        merge_css(&all_css.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }

    pub fn get_handler(&self, event: &str) -> Option<&EventHandler> {
        self.handlers.get(event)
    }

    pub fn has_handler(&self, event: &str) -> bool {
        self.handlers.contains_key(event)
    }

    pub fn trigger_handler(&self, event: &str) {
        if let Some(handler) = self.handlers.get(event) {
            handler();
        }
    }

    pub fn get_attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

impl VContent {
    pub fn string(s: String) -> Self {
        Self {
            string: Some(s),
            image_path: None,
        }
    }

    pub fn image(path: String) -> Self {
        Self {
            string: None,
            image_path: Some(path),
        }
    }
}

#[macro_export]
macro_rules! components_list {
    ($($name:ident),* $(,)?) => {
        $(
            fn $name() -> VComponent {
                VComponent::from_str(include_str!(concat!(
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

// Base CSS for elements generated by VComponent::render().
// Kept extremely small and compatible with our selector matcher.
fn base_css() -> &'static str {
    ".vk-img{width:100%;height:100%;}"
}

fn replace_first_content_image(input: &str, replacement: &str) -> String {
    for pat in [
        "<Content type=\"Image\" />",
        "<Content type=\"Image\"/>",
        "<Content type='Image' />",
        "<Content type='Image'/>",
        "<Content type=\"image\" />",
        "<Content type=\"image\"/>",
        "<Content type='image' />",
        "<Content type='image'/>",
    ] {
        if let Some(pos) = input.find(pat) {
            let mut out = String::with_capacity(input.len() - pat.len() + replacement.len());
            out.push_str(&input[..pos]);
            out.push_str(replacement);
            out.push_str(&input[pos + pat.len()..]);
            return out;
        }
    }
    input.to_string()
}

fn replace_first_content_text(input: &str, replacement: &str) -> String {
    for pat in [
        "<Content type=\"Text\" />",
        "<Content type=\"Text\"/>",
        "<Content type='Text' />",
        "<Content type='Text'/>",
        "<Content type=\"text\" />",
        "<Content type=\"text\"/>",
        "<Content type='text' />",
        "<Content type='text'/>",
    ] {
        if let Some(pos) = input.find(pat) {
            let mut out = String::with_capacity(input.len() - pat.len() + replacement.len());
            out.push_str(&input[..pos]);
            out.push_str(replacement);
            out.push_str(&input[pos + pat.len()..]);
            return out;
        }
    }
    input.to_string()
}

fn escape_attr_value(s: &str) -> String {
    // Minimal attribute escaping for our generated HTML.
    s.replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
