//! 最小独立应用：界面在 `hello.json`，Rust 只保留状态与事件。
//!
//! 前端：`examples/hello.json`（标题、窗口属性、页面树与控件）。
//! 后端：本文件（状态、主题切换、按钮事件、消息对话框）。

use cloud_ui::{
    AppEvent, AppLayout, Application, CLOSE_ACTION, Context, MessageDialog, Theme, View,
    WindowOptions, run_native,
};
use cloud_ui_render::Document;
use serde_json::json;

struct Hello {
    layout: AppLayout,
    pointer: Option<(f32, f32)>,
    light: bool,
}

impl Application for Hello {
    fn update(&mut self, ctx: &mut Context) {
        for event in ctx.events() {
            match event {
                AppEvent::MouseMoved { x, y } => self.pointer = Some((*x, *y)),
                AppEvent::MouseLeft => self.pointer = None,
                AppEvent::RegionClicked(name) if name == "theme" => self.light = !self.light,
                AppEvent::RegionClicked(name) if name == "hello" => {
                    let mut dialog = MessageDialog::new(
                        "Hello",
                        "这是独立 CloudUI workspace 里的消息对话框。",
                        "success",
                        &["好"],
                    );
                    // 对话框是类：属性在 show 之前改。
                    dialog.style.text_size = Some(15.0);
                    dialog.show();
                }
                AppEvent::RegionClicked(name) if name == CLOSE_ACTION => ctx.close(),
                AppEvent::Key {
                    vk: 0x1B,
                    pressed: true,
                } => ctx.close(),
                _ => {}
            }
        }
    }

    fn view(&mut self, ctx: &Context) -> Document {
        let theme = if self.light {
            Theme::light()
        } else {
            Theme::dark()
        };
        let (width, height) = ctx.size();
        let mut view = View::new(width, height);
        if let Some((x, y)) = self.pointer {
            view.pointer(x, y);
        }
        // 前端界面来自 JSON；这里只准备这一帧的数据。
        let data = json!({
            "theme_label": if self.light { "深色" } else { "浅色" },
        });
        self.layout
            .build(&mut view, &theme, &data, ctx.is_maximized());
        view.build(|text, slot, size| ctx.measure_text(text, slot, size))
    }
}

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/hello.json");
    let layout = match AppLayout::load(path) {
        Ok(layout) => layout,
        Err(error) => {
            eprintln!("读布局文件失败：{error}");
            return;
        }
    };
    let options = WindowOptions {
        title: layout.window().title.clone(),
        size: layout.window().size,
        decorations: layout.window().decorations,
        caption_height: layout.window().caption_height,
        corner_radius: layout.window().corner_radius,
        ..Default::default()
    };
    if let Err(error) = run_native(
        options,
        Hello {
            layout,
            pointer: None,
            light: false,
        },
    ) {
        eprintln!("cloud-ui 示例启动失败：{error}");
    }
}
