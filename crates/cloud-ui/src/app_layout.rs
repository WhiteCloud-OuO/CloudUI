//! 应用界面文件（JSON）：前端界面与 Rust 逻辑分离。
//!
//! 产品把窗口内容写在一份 JSON 里（标题、尺寸、页面树、控件），Rust 只保留状态与事件处理：
//! 每帧 `AppLayout::build` 把 JSON 展开成 [`View`]，应用自己的 `Application::view` 不再手写控件。
//!
//! 这是**应用侧布局文件**的一个稳定子集，不是完整的 v2 页面树引擎：当前支持
//! `page` / `stem` / `panel` / `text` / `text_button` / `text_edit` / `text_box` / `spacer`，
//! 布局支持 `flex` 的横 / 纵方向、`gap` 与统一 `padding`。未识别的节点 / 字段直接报错，
//! 不会静默丢掉界面。完整页面树契约见 `docs/design/ui-layout.md`。
//!
//! 文本字段支持 `{{path.to.value}}` 数据绑定，值来自每帧传入的 `serde_json::Value`；
//! 缺失字段记一条警告并按空串处理。

use std::fmt;
use std::path::Path;

use cloud_ui_core::Edges;
use serde_json::Value;

use crate::app::{Decorations, WindowOptions};
use crate::view::{StackStyle, Theme, View};

/// 布局文件最大字节数。
const MAX_LAYOUT_BYTES: u64 = 4 * 1024 * 1024;

/// 应用界面文件。
#[derive(Debug, Clone)]
pub struct AppLayout {
    window: WindowSpec,
    page: Node,
}

/// 文件里的窗口属性。
#[derive(Debug, Clone)]
struct WindowSpec {
    title: String,
    size: (f32, f32),
    decorations: Decorations,
    title_bar: bool,
    caption_height: Option<f32>,
    corner_radius: f32,
}

/// 布局节点。
#[derive(Debug, Clone)]
enum Node {
    Stack {
        direction: Direction,
        gap: f32,
        padding: Edges,
        children: Vec<Node>,
    },
    Panel {
        padding: Option<f32>,
        children: Vec<Node>,
    },
    Text {
        text: String,
        style: TextStyle,
    },
    Button {
        name: String,
        text: String,
    },
    TextEdit {
        name: String,
        text: String,
    },
    TextBox {
        name: String,
        text: String,
        height: f32,
    },
    Spacer {
        width: f32,
        height: f32,
    },
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy)]
enum TextStyle {
    Normal,
    Muted,
    Title,
}

/// 布局文件错误。
#[derive(Debug)]
pub struct LayoutError {
    message: String,
}

impl LayoutError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for LayoutError {}

impl AppLayout {
    /// 读一份 JSON 布局文件。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LayoutError> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path).map_err(|error| {
            LayoutError::new(format!("读不了布局文件 {}：{error}", path.display()))
        })?;
        if metadata.len() > MAX_LAYOUT_BYTES {
            return Err(LayoutError::new(format!(
                "布局文件 {} 太大（{} 字节，上限 {MAX_LAYOUT_BYTES}）",
                path.display(),
                metadata.len()
            )));
        }
        let text = std::fs::read_to_string(path).map_err(|error| {
            LayoutError::new(format!("读不了布局文件 {}：{error}", path.display()))
        })?;
        Self::parse(&text).map_err(|error| {
            LayoutError::new(format!("布局文件 {}：{}", path.display(), error.message))
        })
    }

    /// 从 JSON 文本解析。
    pub fn parse(text: &str) -> Result<Self, LayoutError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let value: Value = serde_json::from_str(text)
            .map_err(|error| LayoutError::new(format!("不是合法 JSON：{error}")))?;
        let root = value
            .as_object()
            .ok_or_else(|| LayoutError::new("根必须是对象"))?;
        if let Some(version) = root.get("version").and_then(Value::as_u64)
            && version != 2
        {
            return Err(LayoutError::new(format!(
                "只支持 version: 2（应用布局子集），读到 {version}"
            )));
        }

        let window = root
            .get("window")
            .map(parse_window)
            .transpose()?
            .unwrap_or_else(default_window);
        let page = root
            .get("page")
            .ok_or_else(|| LayoutError::new("缺 page 节点"))?;
        let page = parse_node(page)?;
        Ok(Self { window, page })
    }

    /// 文件里的窗口属性，转成 [`WindowOptions`]。
    pub fn window(&self) -> WindowOptions {
        let mut options = WindowOptions {
            title: self.window.title.clone(),
            size: self.window.size,
            decorations: self.window.decorations,
            caption_height: self.window.caption_height,
            corner_radius: self.window.corner_radius,
            ..Default::default()
        };
        if self.window.decorations == Decorations::None
            && self.window.title_bar
            && options.caption_height.is_none()
        {
            options.caption_height = Some(36.0);
        }
        options
    }

    /// 把 JSON 界面展开到这一帧的 [`View`]。
    ///
    /// `data` 是 `{{...}}` 数据绑定的来源；`maximized` 只影响根背景圆角与标题栏按钮。
    pub fn build(&self, view: &mut View, theme: &Theme, data: &Value, maximized: bool) {
        let radius = if maximized {
            0.0
        } else {
            self.window.corner_radius
        };
        view.background_rounded(theme.background, radius);
        if self.window.decorations == Decorations::None && self.window.title_bar {
            view.title_bar(&self.window.title, theme.title_bar(), maximized);
        }
        self.build_node(view, theme, data, &self.page);
    }

    fn build_node(&self, view: &mut View, theme: &Theme, data: &Value, node: &Node) {
        match node {
            Node::Stack {
                direction,
                gap,
                padding,
                children,
            } => {
                let style = StackStyle {
                    gap: *gap,
                    padding: *padding,
                };
                let add = |ui: &mut View| {
                    for child in children {
                        self.build_node(ui, theme, data, child);
                    }
                };
                match direction {
                    Direction::Row => {
                        view.row(style, add);
                    }
                    Direction::Column => {
                        view.column(style, add);
                    }
                }
            }
            Node::Panel { padding, children } => {
                let mut style = theme.panel();
                if let Some(padding) = padding {
                    style.padding = Edges::all(*padding);
                }
                view.panel(style, |ui| {
                    for child in children {
                        self.build_node(ui, theme, data, child);
                    }
                });
            }
            Node::Text { text, style } => {
                let style = match style {
                    TextStyle::Normal => theme.label(),
                    TextStyle::Muted => theme.muted(),
                    TextStyle::Title => theme.title(),
                };
                view.label(resolve(text, data), style);
            }
            Node::Button { name, text } => {
                view.button(name, resolve(text, data), theme.button());
            }
            Node::TextEdit { name, text } => {
                view.text_edit(name, resolve(text, data), theme.text_edit());
            }
            Node::TextBox { name, text, height } => {
                view.text_box(name, resolve(text, data), theme.text_box(*height));
            }
            Node::Spacer { width, height } => {
                view.spacer(*width, *height);
            }
        }
    }
}

fn default_window() -> WindowSpec {
    WindowSpec {
        title: "CloudUI".to_owned(),
        size: (800.0, 600.0),
        decorations: Decorations::System,
        title_bar: false,
        caption_height: None,
        corner_radius: 12.0,
    }
}

fn parse_window(value: &Value) -> Result<WindowSpec, LayoutError> {
    let object = value
        .as_object()
        .ok_or_else(|| LayoutError::new("window 必须是对象"))?;
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("CloudUI")
        .to_owned();
    let size = match object.get("size") {
        None => (800.0, 600.0),
        Some(value) => {
            let array = value
                .as_array()
                .filter(|array| array.len() == 2)
                .ok_or_else(|| LayoutError::new("window.size 必须是 [宽, 高]"))?;
            let width = number(&array[0], "window.size[0]")?;
            let height = number(&array[1], "window.size[1]")?;
            if width <= 0.0 || height <= 0.0 {
                return Err(LayoutError::new("window.size 必须是正数"));
            }
            (width, height)
        }
    };
    let decorations = match object.get("decorations").and_then(Value::as_str) {
        None | Some("system") => Decorations::System,
        Some("none") => Decorations::None,
        Some(other) => {
            return Err(LayoutError::new(format!(
                "window.decorations 只支持 \"none\" / \"system\"，读到 {other}"
            )));
        }
    };
    let title_bar = object
        .get("title_bar")
        .and_then(Value::as_bool)
        .unwrap_or(decorations == Decorations::None);
    let caption_height = object
        .get("caption_height")
        .map(|value| number(value, "window.caption_height"))
        .transpose()?;
    let corner_radius = object
        .get("corner_radius")
        .map(|value| number(value, "window.corner_radius"))
        .transpose()?
        .unwrap_or(12.0);
    Ok(WindowSpec {
        title,
        size,
        decorations,
        title_bar,
        caption_height,
        corner_radius,
    })
}

fn parse_node(value: &Value) -> Result<Node, LayoutError> {
    let object = value
        .as_object()
        .ok_or_else(|| LayoutError::new("节点必须是对象"))?;
    let kind = object.get("type").and_then(Value::as_str).unwrap_or("stem");
    match kind {
        "page" | "stem" => parse_stack(object),
        "panel" => parse_panel(object),
        "text" | "label" => {
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| LayoutError::new("text / label 节点缺 text"))?;
            let style = match object.get("style").and_then(Value::as_str) {
                None | Some("normal") => TextStyle::Normal,
                Some("muted") => TextStyle::Muted,
                Some("title") => TextStyle::Title,
                Some(other) => {
                    return Err(LayoutError::new(format!(
                        "text.style 只支持 normal / muted / title，读到 {other}"
                    )));
                }
            };
            Ok(Node::Text {
                text: text.to_owned(),
                style,
            })
        }
        "text_button" | "button" => {
            let name = required_name(object)?;
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| LayoutError::new("text_button 缺 text"))?;
            Ok(Node::Button {
                name,
                text: text.to_owned(),
            })
        }
        "text_edit" => {
            let name = required_name(object)?;
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            Ok(Node::TextEdit { name, text })
        }
        "text_box" => {
            let name = required_name(object)?;
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let height = object
                .get("height")
                .map(|value| number(value, "text_box.height"))
                .transpose()?
                .unwrap_or(80.0);
            Ok(Node::TextBox { name, text, height })
        }
        "spacer" => {
            let width = object
                .get("width")
                .map(|value| number(value, "spacer.width"))
                .transpose()?
                .unwrap_or(0.0);
            let height = object
                .get("height")
                .map(|value| number(value, "spacer.height"))
                .transpose()?
                .unwrap_or(0.0);
            Ok(Node::Spacer { width, height })
        }
        other => Err(LayoutError::new(format!(
            "不支持的节点类型 {other}（支持 page / stem / panel / text / label / text_button / button / text_edit / text_box / spacer）"
        ))),
    }
}

fn parse_stack(object: &serde_json::Map<String, Value>) -> Result<Node, LayoutError> {
    let layout = object.get("layout").and_then(Value::as_object);
    let direction = match layout
        .and_then(|layout| layout.get("direction"))
        .and_then(Value::as_str)
    {
        None | Some("column") => Direction::Column,
        Some("row") => Direction::Row,
        Some(other) => {
            return Err(LayoutError::new(format!(
                "layout.direction 只支持 row / column，读到 {other}"
            )));
        }
    };
    let gap = layout
        .and_then(|layout| layout.get("gap"))
        .map(|value| number(value, "layout.gap"))
        .transpose()?
        .unwrap_or(0.0);
    let padding = parse_padding(object, layout)?;
    let children = parse_children(object)?;
    Ok(Node::Stack {
        direction,
        gap,
        padding,
        children,
    })
}

fn parse_panel(object: &serde_json::Map<String, Value>) -> Result<Node, LayoutError> {
    let padding = object
        .get("padding")
        .map(|value| number(value, "padding"))
        .transpose()?;
    let children = parse_children(object)?;
    Ok(Node::Panel { padding, children })
}

fn parse_padding(
    object: &serde_json::Map<String, Value>,
    layout: Option<&serde_json::Map<String, Value>>,
) -> Result<Edges, LayoutError> {
    let padding = object
        .get("padding")
        .or_else(|| layout.and_then(|layout| layout.get("padding")))
        .map(|value| number(value, "padding"))
        .transpose()?
        .unwrap_or(0.0)
        .max(0.0);
    Ok(Edges::all(padding))
}

fn parse_children(object: &serde_json::Map<String, Value>) -> Result<Vec<Node>, LayoutError> {
    let children = object
        .get("children")
        .and_then(Value::as_array)
        .ok_or_else(|| LayoutError::new("stem / panel / page 缺 children 数组"))?;
    children.iter().map(parse_node).collect()
}

fn required_name(object: &serde_json::Map<String, Value>) -> Result<String, LayoutError> {
    object
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| LayoutError::new("控件缺 name"))
}

fn number(value: &Value, field: &str) -> Result<f32, LayoutError> {
    value
        .as_f64()
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
        .ok_or_else(|| LayoutError::new(format!("{field} 必须是数字")))
}

/// 把 `{{path.to.value}}` 换成数据里的值；缺失字段按空串处理并记警告。
fn resolve(text: &str, data: &Value) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let path = after[..end].trim();
        if let Some(value) = lookup(data, path) {
            output.push_str(&value);
        } else {
            tracing::warn!(path, "布局数据缺字段，按空串处理");
        }
        rest = &after[end + 2..];
    }
    output.push_str(rest);
    output
}

fn lookup(data: &Value, path: &str) -> Option<String> {
    let mut value = data;
    for part in path.split('.').filter(|part| !part.is_empty()) {
        value = match value {
            Value::Object(object) => object.get(part)?,
            Value::Array(array) => array.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_minimal_layout() {
        let layout = AppLayout::parse(
            r##"{
                "version": 2,
                "window": { "title": "T", "size": [320, 200], "decorations": "none" },
                "page": {
                    "layout": { "direction": "row", "gap": 8, "padding": 12 },
                    "children": [
                        { "type": "label", "text": "hi {{name}}" },
                        { "type": "text_button", "name": "go", "text": "Go" }
                    ]
                }
            }"##,
        )
        .expect("layout");
        let options = layout.window();
        assert_eq!(options.title, "T");
        assert_eq!(options.size, (320.0, 200.0));
        assert_eq!(options.caption_height, Some(36.0));
    }

    #[test]
    fn resolves_nested_data() {
        let data = serde_json::json!({ "user": { "name": "Ada" }, "n": 3 });
        assert_eq!(resolve("{{ user.name }} / {{n}}", &data), "Ada / 3");
        assert_eq!(resolve("{{missing}}", &data), "");
        assert_eq!(resolve("plain", &data), "plain");
    }

    #[test]
    fn rejects_unknown_nodes() {
        let error = AppLayout::parse(
            r#"{
                "page": { "children": [{ "type": "chart" }] }
            }"#,
        )
        .expect_err("unknown node");
        assert!(error.to_string().contains("不支持的节点类型"));
    }
}
