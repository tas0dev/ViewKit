use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use cssparser::{Parser, ParserInput, ToCss};
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use toml::Value;

#[derive(Debug, Clone)]
struct CssDeclBuild {
    selector: String,
    property: String,
    value: String,
}

#[derive(Debug)]
struct ComponentBuild {
    name: String,
    root_tag: String,
    class_name: String,
    hx_get: Option<String>,
    hx_post: Option<String>,
    declarations: Vec<CssDeclBuild>,
}

fn find_project_root(manifest_dir: &Path) -> PathBuf {
    if let Ok(workspace_dir) = env::var("CARGO_WORKSPACE_DIR") {
        return PathBuf::from(workspace_dir);
    }
    for ancestor in manifest_dir.ancestors() {
        if ancestor.join("ramfs").join("Libraries").exists() {
            return ancestor.to_path_buf();
        }
    }
    manifest_dir.to_path_buf()
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = Path::new(&manifest_dir);

    // Host PoC: when MOCHI_HOST_POC is set, skip mochi-specific linker arguments
    if env::var("MOCHI_HOST_POC").is_ok() {
        generate_component_templates(manifest_path);
        return;
    }

    let project_root = find_project_root(manifest_path);
    let libs_dir = project_root.join("ramfs").join("Libraries");

    println!("cargo:rustc-link-search=native={}", libs_dir.display());
    println!("cargo:rustc-link-arg={}/crt0.o", libs_dir.display());
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-arg=-no-pie");
    println!("cargo:rustc-link-arg=-T{}/linker.ld", manifest_dir);
    println!("cargo:rustc-link-arg=--allow-multiple-definition");
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=g");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rustc-link-lib=static=nosys");
    // Ensure libunwind is linked so symbols like _Unwind_GetIP are resolved
    println!("cargo:rustc-link-lib=static=unwind");
    // Also pass libunwind.a directly to the linker to ensure symbols are present
    println!("cargo:rustc-link-arg={}/libunwind.a", libs_dir.display());
    // Link libextra.a which provides minimal getcwd implementation used by libstd
    println!("cargo:rustc-link-arg={}/libextra.a", libs_dir.display());

    let libgcc_s = libs_dir.join("libgcc_s.a");
    let libg = libs_dir.join("libg.a");
    if !libgcc_s.exists() && libg.exists() {
        let tmp = libs_dir.join("libgcc_s.a.tmp");
        if let Err(err) = std::fs::copy(&libg, &tmp) {
            panic!(
                "failed to copy {} to {} for static gcc_s linking: {}",
                libg.display(),
                tmp.display(),
                err
            );
        }
        if let Err(err) = std::fs::rename(&tmp, &libgcc_s) {
            let _ = std::fs::remove_file(&tmp);
            if !libgcc_s.exists() {
                panic!(
                    "failed to rename {} to {} for static gcc_s linking: {}",
                    tmp.display(),
                    libgcc_s.display(),
                    err
                );
            }
        }
    }
    println!("cargo:rustc-link-lib=static=gcc_s");
    println!(
        "cargo:rerun-if-changed={}",
        manifest_path.join("linker.ld").display()
    );
    println!("cargo:rerun-if-changed={}", libs_dir.join("libc.a").display());

    generate_component_templates(manifest_path);
}

fn generate_component_templates(manifest_path: &Path) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let out_path = out_dir.join("components_generated.rs");
    let components_dir = manifest_path.join("src").join("components");
    println!("cargo:rerun-if-changed={}", components_dir.display());

    if !components_dir.is_dir() {
        fs::write(
            &out_path,
            "pub static COMPONENT_TEMPLATES: &[ComponentTemplate] = &[];\n",
        )
            .expect("failed to write empty generated components");
        return;
    }

    let common_css_path = components_dir.join("common.css");
    let index_toml_path = components_dir.join("index.toml");
    println!("cargo:rerun-if-changed={}", common_css_path.display());
    println!("cargo:rerun-if-changed={}", index_toml_path.display());
    let common_css = fs::read_to_string(&common_css_path).unwrap_or_default();
    let common_decls = parse_css_declarations(&common_css);
    let component_names = parse_component_index(&index_toml_path);

    let mut components: Vec<ComponentBuild> = Vec::new();
    for name in component_names {
        let path = components_dir.join(&name);
        let htmx_path = path.join("index.html");
        let css_path = path.join("style.css");
        println!("cargo:rerun-if-changed={}", htmx_path.display());
        println!("cargo:rerun-if-changed={}", css_path.display());
        if !path.is_dir() || !htmx_path.is_file() {
            println!(
                "cargo:warning=component '{}' is listed in index.toml but missing index.html",
                name
            );
            continue;
        }
        let htmx = fs::read_to_string(&htmx_path).unwrap_or_default();
        let css = fs::read_to_string(&css_path).unwrap_or_default();
        let (root_tag, class_name, hx_get, hx_post) = parse_htmx_meta(&htmx);
        let mut declarations = common_decls.clone();
        declarations.extend(parse_css_declarations(&css));
        components.push(ComponentBuild {
            name,
            root_tag,
            class_name,
            hx_get,
            hx_post,
            declarations,
        });
    }

    components.sort_by(|a, b| a.name.cmp(&b.name));
    let generated = emit_generated(&components);
    fs::write(&out_path, generated).expect("failed to write generated components");
}

fn parse_component_index(index_toml_path: &Path) -> Vec<String> {
    let text = match fs::read_to_string(index_toml_path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let value: Value = match text.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let arr = match value.get("components").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in arr {
        if let Some(name) = item.as_str() {
            let n = name.trim();
            if !n.is_empty() {
                out.push(n.to_string());
            }
        }
    }
    out
}

fn parse_htmx_meta(input: &str) -> (String, String, Option<String>, Option<String>) {
    let dom: RcDom = parse_document(RcDom::default(), Default::default()).one(input);
    find_first_element(&dom.document)
        .map(|(tag, class_name, hx_get, hx_post)| (tag, class_name, hx_get, hx_post))
        .unwrap_or_else(|| ("div".to_string(), String::new(), None, None))
}

fn find_first_element(handle: &Handle) -> Option<(String, String, Option<String>, Option<String>)> {
    if let NodeData::Element { name, attrs, .. } = &handle.data {
        let tag = name.local.to_string();
        if tag == "html" || tag == "head" || tag == "body" {
            for child in handle.children.borrow().iter() {
                if let Some(found) = find_first_element(child) {
                    return Some(found);
                }
            }
            return None;
        }
        let attrs = attrs.borrow();
        let mut class_name = String::new();
        let mut hx_get = None;
        let mut hx_post = None;
        for attr in attrs.iter() {
            let k = attr.name.local.to_string();
            let v = attr.value.to_string();
            if k == "class" {
                class_name = v
                    .split_whitespace()
                    .next()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
            } else if k == "hx-get" {
                hx_get = Some(v);
            } else if k == "hx-post" {
                hx_post = Some(v);
            }
        }
        return Some((tag, class_name, hx_get, hx_post));
    }
    for child in handle.children.borrow().iter() {
        if let Some(found) = find_first_element(child) {
            return Some(found);
        }
    }
    None
}

fn parse_css_declarations(css: &str) -> Vec<CssDeclBuild> {
    let mut out = Vec::new();
    for block in css.split('}') {
        let Some((selector_raw, body)) = block.split_once('{') else {
            continue;
        };
        let selector = selector_raw.trim();
        if selector.is_empty() {
            continue;
        }
        for decl in body.split(';') {
            let Some((property_raw, value_raw)) = decl.split_once(':') else {
                continue;
            };
            let property = property_raw.trim();
            if property.is_empty() {
                continue;
            }
            let value_trimmed = value_raw.trim().to_string();
            if value_trimmed.is_empty() {
                continue;
            }
            {
                let mut input = ParserInput::new(value_trimmed.as_str());
                let mut parser = Parser::new(&mut input);
                while let Ok(token) = parser.next_including_whitespace_and_comments() {
                    let _ = token.to_css_string();
                }
            }
            out.push(CssDeclBuild {
                selector: selector.to_string(),
                property: property.to_string(),
                value: value_trimmed,
            });
        }
    }
    out
}

fn emit_generated(components: &[ComponentBuild]) -> String {
    let mut src = String::new();
    src.push_str("// @generated by build.rs\n");
    if components.is_empty() {
        src.push_str("pub static COMPONENT_TEMPLATES: &[ComponentTemplate] = &[];\n");
        return src;
    }

    for (idx, c) in components.iter().enumerate() {
        let _ = writeln!(src, "static DECLS_{}: &[CssDecl] = &[", idx);
        for d in &c.declarations {
            let _ = writeln!(
                src,
                "    CssDecl {{ selector: \"{}\", property: \"{}\", value: \"{}\" }},",
                escape_rust_str(&d.selector),
                escape_rust_str(&d.property),
                escape_rust_str(&d.value)
            );
        }
        src.push_str("];\n");
    }

    src.push_str("pub static COMPONENT_TEMPLATES: &[ComponentTemplate] = &[\n");
    for (idx, c) in components.iter().enumerate() {
        let hx_get = c
            .hx_get
            .as_ref()
            .map(|v| format!("Some(\"{}\")", escape_rust_str(v)))
            .unwrap_or_else(|| "None".to_string());
        let hx_post = c
            .hx_post
            .as_ref()
            .map(|v| format!("Some(\"{}\")", escape_rust_str(v)))
            .unwrap_or_else(|| "None".to_string());
        let _ = writeln!(
            src,
            "    ComponentTemplate {{ name: \"{}\", root_tag: \"{}\", class_name: \"{}\", hx_get: {}, hx_post: {}, declarations: DECLS_{} }},",
            escape_rust_str(&c.name),
            escape_rust_str(&c.root_tag),
            escape_rust_str(&c.class_name),
            hx_get,
            hx_post,
            idx
        );
    }
    src.push_str("];\n");
    src
}

fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\"', "\\\"")
}
