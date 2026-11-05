# 文本编辑宿主（windowless RichEdit）

## 0. 定位

`text_edit` 模块把「文本输入」交给系统的 windowless RichEdit（Msftedit），由它负责：
插入符、选区、双击 / 三击选词选行、剪贴板、内部滚动与系统文本服务。
`cloud-ui` 只负责宿主窗口、位置、颜色、焦点与消息转发；文本内容仍以应用状态为准。

`View::text_edit` / `View::text_box` 画背景与边框，真正的输入层盖在元素的 `edit_rect()` 上。

## 1. 宿主窗口

- 每个编辑框一个 `WS_POPUP | WS_EX_TOOLWINDOW` 宿主窗口，owner 是应用窗口或对话框；
- 显示用 `SW_SHOWNA`，不抢前台；需要输入时由应用或对话框显式 `focus()`；
- `top_most` 跟随所属窗口；不置顶时 `raise()` 用 `HWND_TOP`，不会浮到别的应用上面；
- 字体、字号、背景、文字色、选区色、插入符色由样式给出；颜色变化时按颜色重建宿主；
- 单行宿主的窗口高度收成一行（行高 + 4）并在外框中垂直居中，避免 RichEdit 默认多行绘制把文字顶到上方；
- 多行宿主先 `ensure_layout()` 激活排版，再把真实滚动范围回写到 `TextBox.h_scroll` / `v_scroll`，
  渲染器才能画内部滚动条。

## 2. 几何与 DPI

- 应用每帧生成 `Document`，`sync_text_edits` 同时处理 `text_edits()` 与 `text_boxes()`；
- 宿主位置 = 内容区坐标 × DPI + 阴影白边，再 `ClientToScreen`；
- 滚动容器里的编辑框按 placement 的 `clip` 收窄宿主矩形；完全滚出视口就隐藏；
- 窗口移动 / 对话框拖拽时，宿主要跟着平移：`move_by(dx, dy)` 直接改窗口位置并同步内部 bounds 缓存，
  避免下一次 `set_bounds` 再按新坐标搬一次而二次位移；
- 宿主是独立顶层窗口，父窗口移动不会带着它走；普通窗口移动靠下一帧 `sync_text_edits` 对齐，
  模态期间的对话框拖拽则由对话框按相同位移直接搬运。

## 3. 单行与多行

| | 单行 `text_edit` | 多行 `text_box` |
|---|---|---|
| 高度 | 一行（行高 + 4） | 由样式给固定高度 |
| 换行 | 不换行；`\n` 按字面量处理 | 自动换行或按文本硬换行 |
| 滚动 | 无内部滚动条 | RichEdit 自己维护，回写滚动状态给渲染器 |
| 回车 | 交回应用（`TextKey`） | 插入换行 |

## 4. 事件与数据流

- `AppEvent::TextChanged { name, text }`：文本变化（应用状态是唯一真相，下一帧再传回来）；
- `AppEvent::TextFocusChanged { name, focused }`：宿主获得 / 失去焦点；
- `AppEvent::TextKey { name, vk, modifiers }`：宿主没有消费的按键交回应用（例如单行回车、Tab）；
- `Document` 里的文本只是「这一帧的投影」：应用每帧用自身状态重建 `View`，宿主不持有应用数据；
- 主题切换导致颜色变化时，按颜色比较重建宿主，不重建整个窗口。

## 5. 焦点

- `focus()`：把系统前台窗口与键盘焦点交给宿主，并通知文本服务；
- `select_all(true/false)`：全选 / 取消选择；
- `set_input_blocked(true)`：模态期间屏蔽输入 —— 鼠标穿透（`WM_NCHITTEST` 回 `HTTRANSPARENT`，
  `WM_SETCURSOR` / 鼠标消息直接丢弃），如果宿主正有焦点则 `SetFocus(None)`；
- 对话框关闭时取消屏蔽，并按打开前记录的焦点窗口恢复焦点。

## 6. 消息转发

- 键盘、字符、组合输入、鼠标、滚轮、`WM_SIZE`、`WM_SETFONT` 等消息转给 RichEdit 的文本服务；
- `WM_PAINT` 由宿主自己绘制（先擦背景再 `TxDraw`）；
- 鼠标点击在宿主外框内、字符命中之外时也要把插入点落到该位置；
- 双击 / 三击按系统双击时间与距离判定，第三击选整行；
- 选区变化 / 文本变化 / 滚动变化分别通知给应用层。

## 7. 边界

- 宿主只在 Windows 上存在；不提供跨平台文本编辑后端。
- 组合输入与系统文本服务由 RichEdit 自己处理，`cloud-ui` 不做额外实现。
- `View` 还没有封装显式焦点链与 `Tab` 切换；需要时由应用用 `TextKey` 自行处理。
