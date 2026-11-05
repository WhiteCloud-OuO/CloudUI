# 主题

## 1. 范围

`cloudui` 的主题是一组绘制参数，先只做**颜色**；字体、圆角、间距、阴影形状等后续再进同一份文件。

主题只影响画出来的样子，不改变页面树结构、布局规则和事件语义。

## 2. 数据模型

```rust
pub struct Theme {
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub metrics: ThemeMetrics,
}

pub struct ThemeColors {
    pub window: WindowColors,
    pub text: TextColors,
    pub accent: AccentColors,
    pub state: StateColors,
}
```

- `colors` 是稳定契约；`fonts` / `metrics` 可以分批实现。
- 缺省主题由库内置，宿主可以直接用，也可以按名字覆盖。
- 主题名 = 文件名去掉 `.json`；名字比较大小写不敏感，但展示时保留原大小写。

## 3. 文件格式

```json
{
  "colors": {
    "window": {
      "background": "#FFFFFFFF",
      "border": "#14000000",
      "shadow": "#5A000000"
    },
    "text": {
      "primary": "#D8000000",
      "secondary": "#99000000",
      "disabled": "#61000000"
    },
    "accent": {
      "normal": "#FF4A90E2",
      "hover": "#FF357ABD",
      "active": "#FF2C6DA8"
    },
    "state": {
      "error": "#FFD32F2F",
      "warning": "#FFF57C00",
      "success": "#FF388E3C"
    }
  }
}
```

- 颜色一律 `#AARRGGBB`；也接受 `#RRGGBB`（alpha 视为 `FF`）和省略 `#`。
- 缺键回缺省（`#[serde(default)]`）：写 `{}` 也能读，以后加键不会让老主题文件失效。
- 未知键忽略并记 `warn` 日志；类型错误整份主题读取失败。
- 颜色值非法时整份主题读取失败并给出字段路径，不静默退回缺省。

## 4. 目录与优先级

库**不假设**用户目录：

- 内置主题由库随 crate 提供（编译进二进制或放在 crate 的 `themes/`）；
- 宿主可以给一个或多个可写目录；
- 同名主题按「用户目录 > 内置」覆盖；
- 没有同名时全部列出，按名字排序。

```rust
pub struct ThemeSet { /* ... */ }

impl ThemeSet {
    /// 从内置主题起步。
    pub fn with_builtin() -> Self;
    /// 加一个宿主目录；后加的目录优先级更高。
    pub fn add_dir(&mut self, dir: impl Into<PathBuf>);
    /// 重新扫描目录。
    pub fn reload(&mut self) -> Result<(), ThemeError>;
    /// 按名字找主题（可带 / 不带 `.json`，大小写不敏感）。
    pub fn find(&self, name: &str) -> Option<&Theme>;
}
```

- 目录扫描只读 `.json`；损坏的文件跳过并记日志，不影响其它主题。
- 文件变化不自动监听；宿主需要时调用 `reload`。
- 保存由宿主决定路径；`Theme::save(path)` 只负责序列化和写文件。

## 5. 加载与合并

- 主题文件先反序列化成 `Theme`，再与内置缺省值逐字段合并；缺的字段用缺省值。
- 合并按结构递归，不按整份文件覆盖；这样新增字段时老主题仍然可用。
- 主题缓存键是「路径 + mtime + 文件大小」；命中就跳过读盘和解析。
- 读取失败时 `ThemeSet::find` 返回 `None`，调用方使用缺省主题并记日志。

## 6. 与渲染器的关系

- 渲染器只接收最终的 `Theme`，不读文件、不扫描目录。
- 布局阶段使用的间距 / 圆角来自 `ThemeMetrics`；绘制阶段的颜色 / 阴影来自 `ThemeColors`。
- 换主题后重新构建 `Document` 或至少重新渲染一帧；具体粒度由宿主决定。
- 主题预览可以用同一个 `Renderer` 出图，保证和真实窗口一致。

## 7. 后续

- 字体、字号、行距进 `ThemeFonts`。
- 圆角、留白、间距、阴影大小与模糊进 `ThemeMetrics`。
- 渐变、图片背景、装饰边框。
- 主题继承 / 变体（浅色 / 深色）可以在 `Theme` 之上再加一层解析，不改变文件格式的基本结构。

## 8. 相关文档

- 渲染器：[`rendering.md`](rendering.md)
- 窗口模型：[`windows.md`](windows.md)
- 页面树与布局：[`ui-layout.md`](ui-layout.md)
