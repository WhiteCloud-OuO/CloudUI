//! Windows 自绘窗口壳：分层窗口、窗口模板、内置对话框与 RichEdit 宿主。
//!
//! 底层绘制仍由 [`cloud_ui_render`] 负责：这里管窗口、焦点、鼠标穿透、命中测试、滚动与文本编辑，
//! 再把 `Document` 交给渲染器出图，最后经 `UpdateLayeredWindow` 整块贴上。
//!
//! 模块分层：
//!
//! - [`layered`]：预乘 RGBA 位图贴到分层窗口；
//! - [`window_class`] / [`window`] / [`monitor`]：窗口类、模块句柄、鼠标穿透、显示器工作区；
//! - [`painter`]：一份共享渲染器（字体、主题、阴影）；
//! - [`text_edit`]：windowless RichEdit 宿主（独立应用与对话框的文本编辑共用）；
//! - [`auxiliary`] / [`independent_window_template`] / [`auxiliary_window_template`]：
//!   独立窗口与资产窗口的所有权 / 焦点 / 桥接模型；
//! - [`dialog`]：同步模态对话框（消息 / 文本输入），共用布局与阻塞宿主。
//!
//! **线程**：窗口相关类型都在 UI 线程上使用（`Rc` / `RefCell` / `thread_local!`），不实现 `Send`；
//! [`RenderSettings`] 只是数据，可跨线程传递。

#[cfg(windows)]
pub mod app;
#[cfg(windows)]
pub mod app_layout;
#[cfg(windows)]
pub mod auxiliary;
#[cfg(windows)]
pub mod auxiliary_window_template;
#[cfg(windows)]
mod current;
#[cfg(windows)]
pub mod dialog;
#[cfg(windows)]
pub mod independent_window_template;
#[cfg(windows)]
pub mod layered;
#[cfg(windows)]
mod mask;
#[cfg(windows)]
pub mod monitor;
#[cfg(windows)]
pub mod painter;
#[cfg(windows)]
pub(crate) mod text_edit;
#[cfg(windows)]
pub mod view;
#[cfg(windows)]
pub mod window;
#[cfg(windows)]
pub mod window_class;

mod region;
mod render_settings;

#[cfg(windows)]
pub use app::{
    AppEvent, Application, Context, Decorations, MouseButton, TrayEvent, TrayMenuItem, TrayOptions,
    WindowId, WindowOptions, run_native,
};
#[cfg(windows)]
pub use app_layout::{AppLayout, LayoutError};
pub use cloud_ui_core::{
    Align, Axis, Constraints, Edges, GridTrack, Insets, Justify, LayoutKind, LayoutResult, Node,
    NodeId, NodeKind, PageTree, Point, Position, Rect, Size, SizeSpec,
};
pub use cloud_ui_types::{
    BarTheme, CandidateTheme, ItemNumberStyle, LayoutMode, ThemeColor, ThemeError, ThemeFile,
    TipTheme,
};
#[cfg(windows)]
pub use dialog::{DialogStyle, MessageDialog, TextInputDialog};
pub use mask::WindowMask;
pub use render_settings::RenderSettings;
#[cfg(windows)]
pub use view::{
    ButtonStyle, CLOSE_ACTION, IconButtonStyle, LabelStyle, ListBoxState, ListBoxStyle, ListItem,
    MAXIMIZE_ACTION, MINIMIZE_ACTION, PanelStyle, ScrollablePageMetrics, ScrollablePageState,
    ScrollablePageStyle, StackStyle, TextEditStyle, Theme, TitleBarIcons, TitleBarStyle, View,
};
