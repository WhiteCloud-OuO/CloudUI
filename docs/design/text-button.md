# 文本按钮（text_button）

状态：v1 已实现（解析、布局、绘制、点击、默认键绑定）。

## 1. 定位

`text_button` 是带文本内容的动态控件，由三部分组成：

1. **背景形状**：决定按钮外观和命中区域；
2. **区域**：由背景形状直接生成，不单独配置；
3. **文本**：默认在背景形状正中央，画在最上层。

它和扁平按钮一样参与页面树布局、可以绑定事件；另外增加了阴影、文本样式和默认按钮绑定。
`text_button` 不继承扁平按钮的 SVG 部分。

## 2. 数据模型

```rust
pub struct TextButton {
    pub name: String,
    pub text: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub background: ButtonBackground,
    pub normal: ButtonStateColors,
    pub hovered: ButtonStateColors,
    pub pressed: ButtonStateColors,
    pub disabled: ButtonStateColors,
    pub show_shadow: bool,
    pub shadow: ButtonShadow,
    pub text_color: Color,
    pub font: ButtonFont,
    pub enabled: bool,
    pub default_button: Option<DefaultButton>,
    pub events: Events,
}

pub struct ButtonBackground {
    pub shape: ButtonShape,
    pub radius: f32,
    /// 正常状态的缺省背景色；`normal.background` 可覆盖。
    pub color: Color,
}

pub struct ButtonStateColors {
    pub background: Color,
    pub text: Color,
}

pub struct ButtonShadow {
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct ButtonFont {
    pub family: Option<String>,
    pub size: f32,
    pub weight: u16,
    pub italic: bool,
}

pub enum DefaultButton {
    Escape,
    Enter,
}
```

- `background.shape`：`rect` / `round_rect` / `circle` / `path`。
- 背景缺省为 `round_rect`，`radius = 3`。
- 四态颜色：`normal` / `hovered` / `pressed` / `disabled`，每态各有 `background` 与 `text`；
  未写的状态沿用正常态颜色。`background.color` / `text_color` 是正常态的缺省值。
- 状态优先级：`enabled = false` → `disabled`；否则按下 → `pressed`、悬停 → `hovered`、其余 → `normal`。
- `show_shadow` 缺省 `false`；只有为 `true` 时才画 `shadow`。
- `shadow` 缺省颜色 `#5A000000`，位置为右下（`offset_x = 2`、`offset_y = 6`，与现有浮层面板的阴影观感一致）；
  模糊半径由主题提供，不放进每个按钮。
- `enabled` 缺省 `true`；为 `false` 时禁用按钮，不响应鼠标 / 键盘，绘制 `disabled` 颜色。
- `default_button` 缺省 `None`。
- `font.weight` 使用 100–900；`italic` 控制斜体。字体属性先只做字重和斜体，下划线 / 删除线等后续再加。
- `font.size` 单位是逻辑点；实际像素按窗口 DPI 换算。
- 所有有边框控件共用 `the_gap_between_border_and_content`（缺省 2 物理像素）；`text_button`
  没有边框时忽略该属性，后续启用边框再按同一规则生效。

## 3. 尺寸与文本

- `w` / `h` 是按钮的**最小尺寸**，不是固定尺寸。
- 实际尺寸按两个来源取较大值：
  1. 显式 `w` / `h`；
  2. 文本测量结果加默认内边距（缺省左右各 12、上下各 6）。
- 两个轴都省略时完全按文本测量自动撑开；只写一个轴时，该轴仍作为最小值。
- 文本默认水平、垂直居中，位于背景形状内容区。
- **背景形状必须完整覆盖文本范围**：最终按钮尺寸至少要装下“文本范围 + 内边距”，
  不能让文本超出背景或压到相邻控件。文本因此不会被省略号截断；页面级内容限额仍然有效。
- v1 文本支持显式换行（`\n`），不做自动换行、不自动缩放；如果需要更大按钮，用 `w` / `h` 给最小值。
- 背景形状是圆角矩形时，圆角半径为 `radius`；`rect` 忽略 `radius`。
- `circle` 的最终宽高相等；要完整覆盖多行文本矩形，直径按文本矩形的对角线加内边距计算。
- `path` 背景按按钮框缩放；最终尺寸要保证多行文本矩形的四个角都落在路径的填充区域内，
  不满足时继续扩大按钮，而不是裁掉文本。

### 3.1 换行与转义

- 文本里的**实际换行符** `\n` 表示换行；在 JSON 字符串中按 JSON 规则写作 `\n`。
- 要显示反斜杠本身，JSON 字符串写作 `\\`（Lua / Rust 字符串同理按各自语言的转义规则写）。
- 布局引擎不对文本做第二次反转义：解析后的字符串里有换行就换行，有单个反斜杠就显示单个反斜杠。
- 每行单独测量，文本范围取：
  - 宽度：最宽那一行的宽度；
  - 高度：行数 × 字体行高（不含额外的行间距）。
- 多行文本作为一个整体在背景形状中居中；每一行也水平居中。
- 末尾的 `\n` 会多出一个空行，按钮高度跟着增加。
- 除 `\n` 外没有其它特殊换行符；`\r` 按普通字符处理，不做换行。

## 4. 区域与事件

- **区域由背景形状生成**，名字就是控件 `name`；不额外提供 `regions` 字段。
- 命中检测只看背景形状，阴影不算命中范围。
- 鼠标点击区域，或默认键触发时，派 `events.clicked` 对应的处理器。
- 键盘触发只与 `default_button` 有关：`default_button = "enter"` 时按 Enter，`"escape"` 时按 Esc；
  没有绑定该键、按钮禁用、或没有 `clicked` 处理器时不消费按键。
- 事件载荷至少包含：
  - `control`：按钮名；
  - `handler`：事件处理器名；
  - `key`：键盘触发时为虚拟键码（Enter = 13、Esc = 27），鼠标触发时为 `nil`。
- 禁用按钮不产生 hover / press / clicked，也不响应默认键，但仍保留自己的区域几何。
- 区域和背景使用同一套几何；圆角 / 圆 / 路径的边缘外不算命中。

## 5. 绘制顺序

同一个按钮的绘制顺序固定为：

1. 阴影（仅 `show_shadow = true`）；
2. 背景形状；
3. 文本。

文本始终在背景上方。阴影画在背景外沿，不改变布局尺寸，也不影响命中区域。

## 6. 默认按钮绑定

`default_button` 可选：

| 值 | 行为 |
|---|---|
| `null`（缺省） | 不响应页面默认键 |
| `"escape"` | 页面上按 `Esc` 时触发按钮 |
| `"enter"` | 页面上按 `Enter` 时触发按钮 |

规则：

- **一个 Page 上最多只能有一个按钮设置非空 `default_button`**。
- 统计范围是**整棵页面树实例**（含子页面实例、列表实例），不是只算主页面直接子节点；也不因按钮暂时隐藏而少算。
- 如果出现第二个非空绑定：
  - 页面可以正常加载；
  - **所有**非空 `default_button` 都失效，记一条错误日志；
  - 按钮仍可用鼠标点击。
- 键盘触发与鼠标点击都走 `clicked` 处理器；没有处理器、按钮禁用、或没有绑定该键时，该键不消费，
  继续按页面常规键盘处理。
- 键盘触发的优先级高于页面方向键 / 翻页等普通处理；默认按钮在键盘分发阶段先处理，触发后消费该键。
- 带 Ctrl / Alt / Shift 的 Enter / Esc 不触发默认按钮。

## 7. JSON 契约

```json
{
  "type": "text_button",
  "name": "ok",
  "w": 96,
  "h": 32,
  "text": "确定",
  "text_color": "#FF1F2329",
  "font": {
    "family": "system-ui",
    "size": 14,
    "weight": 500,
    "italic": false
  },
  "background": {
    "shape": "round_rect",
    "radius": 3,
    "color": "#FFFFFFFF"
  },
  "normal": {
    "background": "#FFFFFFFF",
    "text": "#FF1F2329"
  },
  "hovered": {
    "background": "#FFF2F4F7",
    "text": "#FF1F2329"
  },
  "pressed": {
    "background": "#FFE3E8EF",
    "text": "#FF1F2329"
  },
  "disabled": {
    "background": "#FFE8E8E8",
    "text": "#FFAAAAAA"
  },
  "show_shadow": true,
  "shadow": {
    "color": "#5A000000",
    "offset": { "x": 2, "y": 6 }
  },
  "enabled": true,
  "default_button": "enter",
  "events": {
    "clicked": "ok"
  }
}
```

- 颜色统一 `#AARRGGBB`，也接受 `#RRGGBB`（alpha 视为 `FF`）。
- 文本换行和反斜杠按 JSON 字符串转义：
  - `"第一行\n第二行"` → 两行文本；
  - `"C:\\Temp"` → 显示 `C:\Temp`；
  - `"\\n"` → 显示字面量 `\n`，不是换行。
- `default_button` 不写或写 `null` 表示不绑定。
- `enabled` / `show_shadow` 不写时分别是 `true` / `false`。
- 四态表都可省略；省略的状态沿用正常态颜色。状态表里的 `text` 也接受 `text_color` / `svg` 别名。
- `background` / `shadow` 可省略，使用缺省值。
- `font` 可省略，使用主题正文样式。
- `events` 沿用动态控件的键值表；`clicked` 是鼠标点击和默认键共用的动作。

## 8. 校验

- `name` 非空，同一作用域内唯一。
- `shape` 必须是四种之一；`circle` 最终宽高必须相等。
- `w` / `h` 是最小值；写得更小也会被“文本范围 + 内边距”撑大，不按错误处理。
- `radius` 不能为负；`rect` 下可忽略。
- 颜色值必须合法。
- `show_shadow` / `enabled` 必须是布尔值。
- 四态颜色表可省略；写了就必须是对象，颜色键合法。
- `font.size > 0`；`weight` 在 100–900。
- `default_button` 只能是 `null` / `"escape"` / `"enter"`；重复绑定按 §6 处理。
- 文本长度沿用页面树的通用内容限额；超出限额按布局失败处理，不偷偷截断文本。

## 9. 与扁平按钮的差异

| 能力 | 扁平按钮 | `text_button` |
|---|---|---|
| 内容 | SVG | 文本 |
| 背景形状 | 固定按钮造型 | 可配置 `rect` / `round_rect` / `circle` / `path` |
| 状态颜色 | 背景 + SVG 三态 | 背景 + 文本四态（含禁用） |
| 阴影 | 主题默认 | `show_shadow` 开关，颜色和位置可配置 |
| 文本颜色 / 字体 | 不适用 | 可配置 |
| 可用状态 | 无 | `enabled` |
| 点击事件 | `mouse_clicked` | `clicked`（鼠标 / 默认键） |
| 默认按键 | 无 | `escape` / `enter` / `null` |

## 10. 后续

- 自定义文本内边距、自动换行、字号自适应。
- 默认按钮的焦点框与键盘导航。
- 字体下划线 / 删除线 / 字距。
