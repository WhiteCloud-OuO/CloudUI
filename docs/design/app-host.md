# 独立应用宿主（application host）

## 0. 定位

`cloud-ui` 是 **Windows 专用的自绘 UI 库**，不引入 macOS / Linux 后端。它可以直接承载普通 exe：
一块自绘窗口 + 一个 `Application`，不依赖任何宿主产品。

本文只描述库本身的契约；产品逻辑、配置、数据与业务交互属于产品仓库。
代码路径按本仓库约定写 `cloudui::`，当前实现位于 `cloud-ui` crate，模块名一致。

## 1. 分层

```text
cloud-ui-core    页面树、布局、尺寸（无 Windows 依赖）
渲染器            Document -> 位图（字体、主题、SVG、图片）
cloud-ui         窗口壳、消息循环、Painter、输入、命中、文本宿主
  ├─ app/         普通应用窗口宿主
  ├─ view/        控件树 + Theme
  ├─ text_edit/   windowless RichEdit 宿主
  ├─ layered/     分层窗口贴图
  ├─ auxiliary/   辅助窗口 / 资产 / 焦点
  └─ dialog/      内置对话框（消息 / 文本输入）
```

`cloud-ui` 默认特性为空；可选特性由产品显式打开。独立 exe 不要求额外运行时。

## 2. 入口与对象

```rust
pub fn run_native(options: WindowOptions, app: impl Application) -> Result<()>;

pub trait Application {
    fn update(&mut self, ctx: &mut Context);
    fn view(&mut self, ctx: &Context) -> Document;
}
```

- `update`：消费这一帧之前累积的事件；应用在这里改自己的状态、开关窗口、打开对话框。
- `view`：按当前状态生成一份声明式 `Document`；`cloud-ui` 负责渲染、命中与焦点，不持有应用数据。
- 两者都在 UI 线程上调用，`Application` 不要求 `Send`。

### 2.1 `WindowOptions`

| 字段 | 缺省 | 意思 |
|---|---|---|
| `title` | `"Cloud UI"` | 窗口标题（系统边框时显示，托盘提示缺省也用它） |
| `size` | `(800, 600)` | 初始客户区尺寸（点） |
| `min_size` | `Some((320, 240))` | 最小客户区尺寸；`None` = 不限 |
| `decorations` | `Decorations::System` | 系统边框 / 标题栏，或完全自绘的无边框分层窗口 |
| `top_most` | `false` | 置顶 |
| `shadow` | `true` | 无边框时在内容四周画阴影白边 |
| `corner_radius` | `12.0` | 无边框时的圆角（点），同时写进主题 |
| `caption_height` | `None` | 无边框自绘标题栏的拖动区高度（点） |
| `tray` | `None` | 创建时注册的通知区域图标与菜单 |
| `mask` | `None` | 窗口蒙版的初始值 |
| `render_settings` | 库缺省 | 字体、主题等渲染参数 |

### 2.2 `Context`

`Context` 是 `update` / `view` 期间能碰的那点东西：

- `size()` / `dpi()` / `is_maximized()`、`measure_text(text, slot, size)`；
- `request_redraw()`、`close()`、`minimize()`、`toggle_maximize()`；
- `show()` / `hide()`（托盘「显示 / 隐藏」用）；
- `set_mask(Option<WindowMask>)`；
- `open_window(WindowOptions, app) -> Option<WindowId>`。

窗口命令（最小化 / 最大化 / 关闭 / 显示 / 隐藏）只排队，等当前 `WM_PAINT` 的借用结束后再执行；
否则 `ShowWindow` 会同步发 `WM_SIZE`，重入窗口状态。

### 2.3 `AppEvent`

| 事件 | 意思 |
|---|---|
| `Resized { width, height }` | 客户区尺寸变化（点） |
| `MouseMoved { x, y }` / `MouseLeft` | 指针移动 / 离开客户区 |
| `MouseButton { button, pressed, x, y }` | 左 / 右 / 中键按下或抬起 |
| `RegionClicked(name)` | 左键在 `View` 的区域里按下并松开；`name` 是按钮 / 行 / 区域名 |
| `MouseWheel { delta }` | 滚轮，`120` 为一格 |
| `Key { vk, pressed }` | 虚拟键按下 / 抬起（Win32 键码） |
| `Text(String)` | 字符输入 |
| `TextChanged { name, text }` | 某个 `text_edit` / `text_box` 的文本变化 |
| `TextFocusChanged { name, focused }` | 文本宿主获得 / 失去焦点 |
| `TextKey { name, vk, modifiers }` | 文本宿主里的按键（宿主没消费时交回应用） |
| `Tray(TrayEvent)` | 托盘单击 / 双击 / 右键 / 菜单项 |
| `Focused(bool)` | 窗口获得 / 失去焦点 |

## 3. 窗口后端

### 3.1 系统边框

`Decorations::System` 注册普通 `WS_OVERLAPPEDWINDOW | WS_EX_APPWINDOW` 窗口；
`WM_PAINT` 把 `Document` 渲染成位图后用 `StretchDIBits` 贴到客户区。DPI 按 PerMonitorV2 处理，
`WM_DPICHANGED` 时按系统建议矩形重排。

### 3.2 无边框分层窗口

`Decorations::None` 使用 `WS_POPUP | WS_EX_LAYERED`：

- 窗口尺寸 = 内容尺寸 + 两侧阴影白边；位图包含白边，`UpdateLayeredWindow` 的目标点是**窗口左上角**，
  传内容区坐标会让窗口每帧往右下漂一个白边宽度；
- 阴影复用渲染器的 `Shadow::panel()`；圆角与窗口背景同一套几何；
- 阴影白边内的 `WM_NCHITTEST` 回 `HTTRANSPARENT`，点到阴影等于点穿；
- 输入坐标统一减去白边再按 DPI 缩放；
- 最大化时阴影白边按 0 处理，内容贴满工作区；还原后恢复。

## 4. 自绘标题栏

`View::title_bar(title, style, maximized)` 是普通控件，不依赖窗口模板：

- 标题 + 右侧三个按钮；动作名是 `MINIMIZE_ACTION` / `MAXIMIZE_ACTION` / `CLOSE_ACTION`，
  应用在 `RegionClicked` 里分别调 `Context::minimize()` / `toggle_maximize()` / `close()`；
- 最小化 / 最大化 / 还原缺省使用**设备像素字形**：线宽统一为 `round(1.2 × DPI/96)` 个像素，
  不经过 SVG 等比缩放，避免「最小化 2px、最大化 1px」；
- 关闭按钮始终使用 SVG；`currentColor` 跟随主题，其它填充色保持源色；
- 设置 `WindowOptions::caption_height` 后，标题栏高度内**没有命中区域**的地方 `WM_NCHITTEST`
  回 `HTCAPTION`（可拖动、双击最大化 / 还原）；落在 `Document.regions` 上的地方回 `HTCLIENT`，
  按钮才能收到点击；标题栏以下照旧 `HTCLIENT`；
- 无边框窗口带标题栏时补 `WS_MINIMIZEBOX | WS_MAXIMIZEBOX`，不加 `WS_THICKFRAME`，
  避免系统边框和系统命中区。

## 5. 多窗口

`Context::open_window` 在同一个 UI 线程上再开一块窗口：

- 线程本地表是 `WINDOWS`（`HWND` → 窗口状态），窗口过程按 `HWND` 找自己那块，不再假设线程上只有一个窗口；
- 每块窗口有自己的 `Application`、`Painter`、`Document`、RichEdit 宿主、标题栏和窗口命令；
- `WindowId` 支持 `is_open` / `close` / `focus` / `is_maximized`；
- 新窗口先登记，等调用它的那一帧 `WM_PAINT` 借用结束后再统一 `ShowWindow`，避免同步焦点消息重入；
- 最后一个窗口销毁时才退出消息循环；销毁时先摘出编辑框宿主，离开借用后再析构。

## 6. 窗口蒙版

窗口蒙版是独立窗口的一项属性，不是控件，也不进 `Document`：

- `WindowMask { color }`：RGB 是蒙版颜色，`a` 是整体不透明度；`WindowMask::dim(0.35)` 是 35% 黑；
- 实现是一块独立的顶层分层窗口（owner = 应用窗口），盖在客户区上方、所有 RichEdit 宿主上方；
- 跟随客户区：`WM_SIZE` / `WM_MOVE` / `WM_DPICHANGED` 重新摆放；无边框时扣掉阴影白边；
  圆角用与窗口背景同一套 `tiny-skia` 几何生成预乘 RGBA 位图，逐像素 `UpdateLayeredWindow` 贴上，
  不用 `SetWindowRgn` 的硬区域，圆角边缘不会漏像素；
- 不参与输入：`WS_EX_TRANSPARENT | WS_EX_NOACTIVATE`，`WM_NCHITTEST` 回 `HTTRANSPARENT`，
  `WM_MOUSEACTIVATE` 回 `MA_NOACTIVATE`；鼠标落到下面的按钮 / 编辑框，键盘焦点不变；
- 蒙版在上面时不再抬 RichEdit 宿主，避免每帧 z-order 来回切导致编辑框背景闪；
- 隐藏 / 最小化时蒙版和 RichEdit 宿主一起收起，恢复后下一帧再同步。

## 7. 通知区域（托盘）

`WindowOptions::tray: Option<TrayOptions>` 在窗口创建时注册图标，窗口销毁时摘掉：

- 图标按 `SM_CXSMICON` 取 exe 内嵌的应用图标，取不到退回系统缺省图标；
- `TrayOptions::new(tooltip).menu(vec![TrayMenuItem::new(id, label), …])` 配右键菜单；
  菜单项只有文字，没有分隔线 / 勾选 / 禁用状态；
- 左键单击 / 双击 / 右键 / 菜单项都以 `AppEvent::Tray(TrayEvent)` 交回应用；
- 显示 / 隐藏窗口由应用决定：`Context::show()` / `hide()` 先排队，借用结束后执行；
- 窗口隐藏或最小化时没有 `WM_PAINT`，托盘消息里会当场跑一遍 `Application::update`
  （只跑 update，不渲染），否则菜单里的「显示窗口 / 退出」不会生效；
- 主窗口不在前台时不能作为弹出菜单的 owner（菜单会一弹就收），这时改用离屏 `WS_POPUP` 小窗做 owner，
  弹完把前台还给原窗口；
- 监听 `TaskbarCreated` 广播，Explorer 重启后自动重新注册。

## 8. 控件层与主题

`View` 是控件树：

- 容器分两类：**控件容器**（`panel`、`scrollable_page`）与**子页面容器**（`list_box`，通用 `SubpageContainer` 预留）；
- 控件：`label`、`button`、`spacer`、`text_edit`、`text_box`，外加整窗 `background`；
- 布局：`cloud-ui-core` 的 measure-arrange，文字尺寸由 `Painter::measure_text` 提供；
- 输出：`View::build(measure)` 生成 `Document`，按钮 / 列表行同时写入 `Document.regions`；
- 主题：`view::Theme` 提供颜色、字号、圆角、按钮高度等，`dark()` / `light()` 两套缺省；
  派生 `title()` / `label()` / `muted()` / `button()` / `panel()` / `list_box(h)` / `text_edit()` /
  `text_box(h)` / `scrollable_page(h)` 等样式。

容器与列表盒的细节见 [`container.md`](container.md)、[`list-box.md`](list-box.md)、[`ui-layout.md`](ui-layout.md)。

### 8.1 界面文件（JSON）与 Rust 逻辑分离

界面可以写在 JSON 里，Rust 只保留状态与事件：`AppLayout::load` 读文件，`AppLayout::window()`
得到窗口选项，`Application::view` 每帧调 `AppLayout::build(&mut view, &theme, &data, maximized)`
展开页面树，再用 `View::build` 出图。文本里的 `{{path}}` 从每帧传入的 `serde_json::Value` 取值。

文件结构、支持的节点、数据绑定与错误处理见 [`app-layout.md`](app-layout.md)；
`crates/cloud-ui/examples/hello.json` + `hello.rs` 是最小示例。

## 9. 文本编辑

`View::text_edit` / `text_box` 生成 `Element::TextEdit` / `Element::TextBox`；真正的输入由
`text_edit` 模块的 windowless RichEdit 宿主负责。宿主生命周期、颜色、滚动、屏蔽与跟随移动见
[`text-edit.md`](text-edit.md)。

## 10. 内置对话框

`MessageDialog` / `TextInputDialog` 是库内置的类：`new` 建实例、改属性、`show(self)` 同步阻塞。
宿主、模态、拖拽、文本输入与属性清单见 [`dialog.md`](dialog.md)。

## 11. 边界与未做

- 只做 Windows；没有 macOS / Linux 后端。
- 控件层尚未封装：通用 `SubpageContainer`、图片控件、列表虚拟化、滚动条拖动、按钮 hover / pressed 状态、
  焦点链。
- 普通窗口的完整语义尚未封装：应用菜单、拖拽缩放、Aero Snap。
- 窗口蒙版只支持纯色半透明；模糊 / 渐变不做。
- 文本编辑尚未接入 `View` 的显式焦点 API 与 `Tab` 焦点切换。
