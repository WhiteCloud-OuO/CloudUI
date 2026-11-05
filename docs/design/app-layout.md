# 应用布局文件（JSON）

## 0. 目标：前后端分离

独立应用的界面写在 **JSON 文件**里，Rust 只保留状态与逻辑：

| 层 | 文件 | 负责 |
|---|---|---|
| 前端 | `*.json` | 窗口标题 / 尺寸、页面树、控件、静态文字与 `{{...}}` 占位 |
| 后端 | `*.rs` | `Application` 状态、事件处理、主题切换、弹对话框、数据来源 |

`Application::view` 不再手写控件树，只准备这一帧的数据并调用
[`AppLayout::build`](../../crates/cloud-ui/src/app_layout.rs)：

```rust
let data = json!({ "user": self.user, "theme_label": ... });
self.layout.build(&mut view, &theme, &data, ctx.is_maximized());
view.build(|text, slot, size| ctx.measure_text(text, slot, size))
```

示例见 `crates/cloud-ui/examples/hello.json` 与 `hello.rs`。

## 1. 加载

```rust
let layout = AppLayout::load("ui/hello.json")?;   // 或 AppLayout::parse(text)
let options = layout.window();                    // WindowOptions
run_native(options, App { layout, ... })?;
```

- 文件上限 4 MB；UTF-8（允许 BOM）；
- 根必须是对象；`page` 必需；可选的 `version` 只接受 `2`；
- 读到不认识的节点类型 / 字段会返回 `LayoutError`，不会静默丢掉界面。

## 2. 文件结构

```json
{
  "version": 2,
  "window": {
    "title": "CloudUI Hello",
    "size": [520, 260],
    "decorations": "none",
    "caption_height": 36,
    "corner_radius": 12
  },
  "page": {
    "layout": { "direction": "column", "gap": 12, "padding": 16 },
    "children": []
  }
}
```

### 2.1 `window`

| 字段 | 缺省 | 意思 |
|---|---|---|
| `title` | `"CloudUI"` | 窗口标题；无边框时也是自绘标题栏文字 |
| `size` | `[800, 600]` | 初始客户区尺寸（点） |
| `decorations` | `"system"` | `"none"` 无边框分层窗口 / `"system"` 系统边框 |
| `title_bar` | `decorations == "none"` | 是否画自绘标题栏 |
| `caption_height` | 标题栏开启时 36 | 标题栏拖动区高度（点） |
| `corner_radius` | 12 | 无边框圆角（点） |

`window()` 会把它们转成 `WindowOptions`；其余窗口选项（托盘、蒙版、渲染设置）仍由 Rust 侧设置。

## 3. 节点

### 3.1 容器

```json
{ "type": "stem", "layout": { "direction": "row", "gap": 8, "padding": 12 }, "children": [] }
{ "type": "panel", "padding": 16, "children": [] }
```

| 类型 | 说明 |
|---|---|
| `page` / `stem` | 无外观的布局分组；`layout.direction` 为 `row` / `column`，`gap`、`padding` 单位点 |
| `panel` | 有主题底色 / 圆角的面板；`padding` 可覆盖主题缺省，子节点按纵向排列（间距 8） |

`page` 本身就是根 `stem`，必须有 `children`。

### 3.2 控件

```json
{ "type": "label", "text": "你好，{{user.name}}" , "style": "muted" }
{ "type": "text_button", "name": "ok", "text": "确定" }
{ "type": "text_edit", "name": "input", "text": "{{user.name}}" }
{ "type": "text_box", "name": "notes", "text": "第一行\n第二行", "height": 80 }
{ "type": "spacer", "height": 12 }
```

| 类型 | 必填 | 说明 |
|---|---|---|
| `text` / `label` | `text` | 单行文字；`style`：`normal`（缺省）/ `muted` / `title` |
| `text_button` / `button` | `name`、`text` | `name` 是 `AppEvent::RegionClicked` 收到的区域名 |
| `text_edit` | `name` | 单行编辑框；`text` 为初始文本 |
| `text_box` | `name` | 多行编辑框；`height` 缺省 80 |
| `spacer` | — | `width` / `height` 缺省 0 |

样式（颜色、字号、圆角、按钮底色）来自应用当前的 `view::Theme`；JSON 只管结构与内容，
所以同一份界面可以直接跑深色 / 浅色主题。

## 4. 数据绑定

- 文本字段支持 `{{path.to.value}}`，路径按 `.` 取对象字段，数字下标取数组元素；
- 值来自 `build` 每帧传入的 `serde_json::Value`；
- 缺失字段记一条 `tracing::warn!` 并按空串处理；
- 不支持表达式、函数调用或嵌套 `{{}}`。

按状态切换文字、按钮标签时，由 Rust 改变量再传入 data，不需要改 JSON 结构。

## 5. 与完整页面树的关系

本文件描述的是**应用侧布局子集**：容器 / 控件 / 纵向横向排列 / 间距与内边距，足够搭普通设置页、
表单和工具面板。完整 v2 页面树（flex / linear / grid / responsive、子页面、列表盒、数据上下文、
状态值等）见 [`ui-layout.md`](ui-layout.md)；它目前由脚本侧使用，应用宿主的 JSON 加载器还没有接
全部节点类型，遇到不支持的节点会明确报错。

## 6. 约定

- 控件 `name` 必须非空；应用按它处理 `RegionClicked` / `TextChanged`。
- 一个 JSON 文件对应一个窗口；多窗口分别加载各自的文件。
- 布局文件只描述界面，不写事件处理、不写业务状态。
- 修改 JSON 不需要改 Rust 代码；只有新增节点类型 / 绑定语义才需要扩展加载器。
