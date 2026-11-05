# 容器（container）

状态：设计初稿。控件容器已有实现（`panel`），子页面容器留待后续实现。

## 1. 定位

容器是页面树里**专门用来组织子节点**的布局节点。按子节点类型分成两类，不混用：

1. **控件容器**（controls container）：放控件，例如静态面板 `panel`。
2. **子页面容器**（subpage container）：只放子页面，后续实现。

“有 `children`”不等于“是容器”：`self_drawing_panel` 也带 `children`，但它是特殊控件，
不参与这里的容器分类和子节点矩阵。

`Stem` 不是容器：它只是抽象分组，用来调整布局和树结构；`Page` 是窗口根，
可以同时容纳控件、`Stem` 和两类容器。

## 2. 类型关系

```text
Container（抽象）
├─ ControlsContainer（控件容器）
│  └─ Panel（静态面板）
└─ SubpageContainer（子页面容器）
   ├─ ListBox（list_box，单列列表盒）
   └─ 其他子页面容器（后续）
```

- `ControlsContainer` 目前的具体控件类型是 `panel`。
- `self_drawing_panel` **不是容器**，是一个带内部绘制列表的特殊控件；不归入本类型树。
- `SubpageContainer` 是子页面容器大类，`list_box` 是它的第一个具体控件；通用
  `subpage_container` 类型保留给后续实现。`list_box` 的完整规则见 [`list-box.md`](list-box.md)。
- 两类容器共享尺寸、定位、可见性、`layout`、`padding`、`margin` 等布局字段；
  差别在**允许的子节点**和**是否有自己的视觉外观**。

## 3. 控件容器

### 3.1 职责

- 负责自己的背景、边框、圆角、内边距和布局。
- 直接持有控件子树；控件名称与普通控件一样参与当前作用域唯一性校验。
- 不创建新的子页面实例作用域；子页面实例作用域只由 `subpage` 创建。

### 3.2 允许的子节点

- `Control`：`label` / `text_button` / `text_edit` / `text_box` / `scroll_bar` 等叶子控件。
- `Stem`：只负责分组和布局。
- `ControlsContainer`：嵌套的控件容器。
- `SubpageContainer`：需要在这个区域里放子页面时使用。

不允许直接放 `Subpage`。子页面必须先放进 `SubpageContainer`，避免“控件树”和
“页面实例树”混在同一层。

### 3.3 外观与命中

- 有背景 / 边框 / 圆角；具体字段由具体控件类型决定。
- 容器本身不因为“是容器”而自动产生点击事件；事件仍由控件、区域或子节点处理。
- 容器背景参与绘制和裁剪，但不自动进入 `regions`。
- 所有有边框控件统一使用：默认边框颜色 `#FFD2D2D2`、默认边框粗细 1 物理像素、
  边框内沿与内容区之间默认 2 物理像素，对应属性名
  `the_gap_between_border_and_content`；`panel`、编辑框、扁平按钮、文本按钮和
  `list_box` 都按这一约定补齐。

### 3.4 自绘面板是特殊控件，不是容器

`self_drawing_panel` 不属于 `ControlsContainer`，也不属于 `SubpageContainer`：

- 它是**控件**，在一个节点内自带 `draw` 列表和局部 `regions`；
- 它的 `children` 是控件内部的组合内容，用来把动态控件画在 `draw` 上，
  不构成页面树里的通用容器边界；
- 不套用第 5 节的容器子节点矩阵，也不能代替 `SubpageContainer` 放子页面；
- 页面树里需要放子页面时，仍然显式使用 `SubpageContainer`。

## 4. 子页面容器

### 4.1 职责

- 只负责给子页面实例分配外框、排列、裁剪和可见性。
- 不拥有业务控件，不直接绘图。
- 子页面根节点自己的背景、边框和内容由子页面负责画。

### 4.2 允许的子节点

- `Subpage`：唯一允许的子节点类型；可以为空（空容器 / 空状态）。
- 不允许 `Control`、`Stem`、`ControlsContainer` 或嵌套 `SubpageContainer`。
- `list_box` 是具体的子页面容器，但使用 `list_box.subpage` 数组声明模板（只用第一个）；
  不是通用容器的 `children` 写法，完整规则见 [`list-box.md`](list-box.md)。

子页面内部仍是一棵完整的 Page 树，可以包含控件容器、控件、Stem 等；
这些属于子页面内部作用域，不是子页面容器的直接子节点。

### 4.3 外观

v1 子页面容器**没有自己的视觉外观**：

- 不接受 `background` / `fill` / `border` / `edge_color` / `radius` / `shadow`。
- 可以有 `layout`、`padding`、`margin`、`w` / `h`、`position`、`visible`、`overflow`。
- 需要背景或边框时，由子页面根节点或外层控件容器提供，避免和子页面背景重复绘制。

后续如果需要标签页底板、分组框等视觉效果，再单独增加属性；不把视觉字段提前塞进 v1。

### 4.4 布局与尺寸

- 子页面容器使用与控件容器相同的布局引擎。
- 子页面实例的外框参与容器的 `linear` / `flex` / `grid` 布局。
- 子页面内部按自己的 Page 根布局测量和排列，不受父容器内部布局影响。
- 子页面容器的 `padding` 作用于所有子页面外框，不作用于子页面内部。
- `w` / `h` 省略时按子页面外框和布局自动撑开；写了就是最小尺寸，最终尺寸不能小于子页面外框。
- `overflow` 缺省为裁剪；子页面内容超出外框时由容器裁剪。

### 4.5 命名与路径

- 子页面容器可以被命名，名称在当前作用域内唯一。
- 子页面实例路径以子页面节点名为基准；如果容器有名字，完整路径包含容器名前缀，
  例如 `pages.rows[0].title`。
- 控件名在实例内部唯一，跨实例允许重名；完整路径用实例路径拼接。

## 5. 允许的子节点矩阵

| 父节点 | Control | Stem | ControlsContainer | SubpageContainer | Subpage |
|---|---:|---:|---:|---:|---:|
| `Page` | 是 | 是 | 是 | 是 | 否 |
| `Stem` | 是 | 是 | 是 | 是 | 否 |
| `ControlsContainer` | 是 | 是 | 是 | 是 | 否 |
| `SubpageContainer` | 否 | 否 | 否 | 否 | 是 |
| `Subpage` 节点 | — | — | — | — | — |

说明：

- `Subpage` 节点在父树里是叶子；它的 `src` 指向的子页面内部另有一棵 Page 树。
- `self_drawing_panel` 按 `Control` 处理；它可以出现在允许控件的位置，但它自己的 `children`
  是控件内部组合内容，不再套用这张矩阵。
- 需要同时放控件和子页面的区域，用 `ControlsContainer` 包一个 `SubpageContainer`，
  或其他布局节点组合；不要在 `SubpageContainer` 里塞控件。
- 旧布局如果直接把 `subpage` 放在 `Page` / `Stem` / `ControlsContainer` 下，
  迁移期可以继续解析并记 `warn`；新写的布局必须使用 `SubpageContainer`。

## 6. JSON 契约

控件容器沿用现有控件类型：

```json
{
  "type": "panel",
  "name": "toolbar",
  "w": 320,
  "h": 48,
  "layout": { "type": "flex", "direction": "row", "gap": 8 },
  "children": [
    { "type": "text_button", "name": "ok", "text": "确定" }
  ]
}
```

子页面容器：

```json
{
  "type": "subpage_container",
  "name": "pages",
  "layout": { "type": "flex", "direction": "column", "gap": 0 },
  "padding": 0,
  "children": [
    {
      "type": "subpage",
      "name": "rows",
      "src": "row.json",
      "repeat": "items",
      "key": "id"
    }
  ]
}
```

- `type`：控件容器为 `panel`（`self_drawing_panel` 是控件，不在此列）；子页面容器为 `subpage_container`。
- `layout`：两类容器都支持；缺省 `flex`。
- `children`：控件容器放控件 / Stem / 容器；子页面容器只放 `subpage`。
- `background` / `border` / `radius` / `shadow`：只允许控件容器。
- `repeat` / `key` / `virtualize` / `required`：仍写在 `subpage` 节点上，不放到容器上。

## 7. 校验

- 容器 `type` 必须是已注册的容器类型；未知类型整份布局失败。
- `SubpageContainer` 的子节点不是 `subpage`：整份布局失败。
- `ControlsContainer` 直接子节点是 `subpage`：迁移期告警并继续；新格式按错误处理。
- 容器名称在当前作用域内唯一；子页面实例使用独立作用域。
- 子页面容器不接受视觉字段，写了按布局失败处理。
- 两类容器的尺寸 / 布局字段缺失按第 4 节规则处理。

## 8. 后续

- 子页面容器的标签页、翻页、懒加载和虚拟化。
- 子页面容器的空状态 / 加载中 / 错误状态。
- 控件容器与子页面容器之间的滚动联动。
- 容器的可访问性名称和焦点顺序。
