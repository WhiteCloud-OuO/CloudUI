# CloudUI

Windows 专用的自绘 UI 库。本仓库同时存放设计文档与可独立构建的 crate 代码：
窗口模型、控件、布局、渲染、主题、文本编辑与内置对话框。

- 只做 Windows，不引入 macOS / Linux 后端。
- 不包含具体宿主产品的业务逻辑；产品怎么用库，写在产品自己的仓库。
- 设计文档入口：[docs/README.md](docs/README.md)。

## 代码

```text
crates/
  cloud-ui-core      页面树、布局约束与尺寸（无 Windows 依赖）
  cloud-ui-types     轻量 UI 数据类型（排布、序号样式、主题）
  cloud-ui-render    自绘渲染器：Document -> 位图
  cloud-ui           窗口壳、应用宿主、控件层、文本编辑、内置对话框
  cloud-ui/examples  hello.json（前端界面）+ hello.rs（后端逻辑）
Images/SVG/          自绘标题栏使用的 SVG 图标
```

`cloud-ui` 默认不带任何可选特性，只提供通用窗口与控件能力；输入法专用的宿主模块、
键位映射参考与脚本窗口不在本仓库范围内。

## 构建

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
