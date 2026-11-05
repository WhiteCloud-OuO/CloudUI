# CloudUI 设计文档

本目录只用于 Rust 库 crate **`cloudui`** 的设计：窗口模型、控件、布局、渲染、主题与公开 API。

**规则**

- 本目录只写 Rust 库 crate `cloudui` 本身的设计。
- 不写具体宿主产品或业务系统的逻辑、配置、数据、交互和实现细节；产品侧怎么用 `cloudui`，写在产品自己的仓库。
- 文档写中文，代码标识符一律英文。实现与文档产生分歧时以 `cloudui` 代码为准，并同步更新文档。
- 当前代码 workspace 的包名是 `cloud-ui`（Rust 路径 `cloud_ui`）；文档里的 `cloudui` 指同一个库。

## 目录

| 目录 | 放什么 |
|---|---|
| [design/](design/) | `cloudui` 的设计：现在是什么样、为什么这么定 |

## 设计文档

| 文件 | 内容 |
|---|---|
| [design/windows.md](design/windows.md) | 自绘窗口模型：独立窗口 / 辅助窗口、所有权、桥接、焦点、资产克隆与窃取、渲染与区域 |
| [design/app-host.md](design/app-host.md) | 独立应用宿主：`run_native`、窗口选项、标题栏、多窗口、蒙版、托盘、控件层与主题 |
| [design/app-layout.md](design/app-layout.md) | 应用布局文件：前端 JSON + 后端 Rust 的分离、文件结构、节点、数据绑定与加载 API |
| [design/dialog.md](design/dialog.md) | 内置对话框：类与属性、消息对话框、文本输入对话框、同步模态、拖拽与输入屏蔽 |
| [design/text-edit.md](design/text-edit.md) | 文本编辑宿主：windowless RichEdit、几何 / DPI、单行与多行、事件、焦点与屏蔽 |
| [design/self-drawing-panel.md](design/self-drawing-panel.md) | 自绘面板（`self_drawing_panel`）：draw / children / 局部 regions、尺寸测量、更新接口与渲染模型 |
| [design/ui-layout.md](design/ui-layout.md) | 页面树：节点模型、linear / flex / grid / responsive 布局、尺寸测量、子页面实例化与旧格式兼容 |
| [design/container.md](design/container.md) | 容器：控件容器（panel 等）与子页面容器的职责、子节点约束、视觉与布局规则 |
| [design/list-box.md](design/list-box.md) | 单列列表盒（`list_box`）：子页面模板、滚动条、选择、事件与数据更新 |
| [design/rendering.md](design/rendering.md) | 自绘渲染器：位图输出、元素、字体、分层窗口与更新策略 |
| [design/theme.md](design/theme.md) | 主题文件：颜色项、文件格式、目录优先级与加载接口 |
| [design/text-button.md](design/text-button.md) | 文本按钮（`text_button`）：背景形状、文本、阴影与页面默认按键绑定 |
