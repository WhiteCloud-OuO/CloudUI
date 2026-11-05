# 自绘面板（self_drawing_panel）

状态：设计定稿，基础模型已明确（draw 元素、JSON 解析、draw / children、裁剪、auto_size、局部 regions、
单元素更新）。自绘面板是独立控件类型，不是 `panel` 的扩展。

## 1. 定位

- `panel`：静态控件容器，只画背景 / 边框，子控件按页面树布局。
- `self_drawing_panel`：**特殊控件**，不是容器；自带内部 draw list 和局部命中区域。
  - `draw` 用基础绘制元素直接画界面，不参与布局；
  - `children` 是控件内部的动态控件组合，参与页面树布局，画在 `draw` 上面；
  - `regions` 给 draw 内容声明局部点击 / 拖拽区域；
  - 默认裁剪在面板范围内；
  - 它不套用 [`container.md`](container.md) 的容器子节点矩阵，也不能代替
    `SubpageContainer` 放子页面。

它同时可用于：

- 页面树 JSON 里的通用控件；
- 由 Rust 直接构造 `Document` 的窗口；
- 列表行、工具条、状态面板等自绘界面的基础。

## 2. JSON 契约

```json
{
  "type": "self_drawing_panel",
  "name": "toolbar",
  "w": 320,
  "h": 240,
  "auto_size": "draw",
  "clip": true,
  "draw": [
    { "name": "bg", "type": "rect", "x": 0, "y": 0, "w": 320, "h": 240,
      "radius": 8, "color": "#FFFFFFFF" },
    { "name": "title", "type": "text", "x": 12, "y": 8,
      "text": "Panel", "size": 14, "font": "ui", "color": "#FF222222" },
    { "name": "icon", "type": "svg", "x": 12, "y": 34, "size": 18,
      "path": "icons/check.svg", "tint": "#FF4A90E2" }
  ],
  "regions": [
    { "name": "row_0", "shape": "rect", "x": 0, "y": 30, "w": 320, "h": 28,
      "drag": false, "events": { "mouse_clicked": "choose" } }
  ],
  "children": []
}
```

### 2.1 draw

`draw` 只接受以下纯绘制元素，拒绝 `label` / `text_block` / 动态控件：

- `rect`
- `line`
- `circle`
- `text`
- `svg`
- `img`
- `path`

每项必须写非空 `name`，同一面板内唯一，且**大小写敏感**（`bg` 和 `BG` 是两个不同的元素）。
缺少 `name`、空 `name` 或完全相同的重名都会让整份布局失败。`name` 只作为 draw 元素标识，
不参与控件事件寻址；事件仍由 `regions` / 面板 `name` 提供。

坐标是面板左上角局部坐标：

- `(0, 0)` = 面板外框左上角；
- 不扣 padding；padding 只影响 `children` 的 content box；
- `draw` 不产生 `event.element`，不单独绑定事件；命中由 `regions` 提供。

### 2.2 children

- `children` 是控件内部的组合内容，放普通控件（可含嵌套自绘面板），参与页面树布局；
  它不是页面树容器边界，不按容器的子节点矩阵扩展；
- 子页面要放进 `SubpageContainer`，不能直接作为 `self_drawing_panel` 的 `children`；
- 位置由页面树与面板的 `layout` / `padding` 决定；
- 绘制顺序：`draw` 在下，`children` 在上；
- 嵌套自绘面板建议限制深度，最多 4 层。

### 2.3 regions

`regions` 是面板局部命中区域，复用通用 `regions` 字段：

- `name`：区域名；同一面板内唯一；
- `shape`：`rect` / `round_rect` / `circle` / `path`；
- `x` / `y` / `w` / `h` / `radius` / `cx` / `cy` / `commands` 等按 shape；
- `drag`：是否可拖动（缺省 `false`）；
- `events`：与动态控件相同的事件表。

事件上下文：

- 面板级 `events` 的 `event.control = 面板的 name`；
- 局部 `regions` 用 `event.region = 局部区域 name` 区分，`event.control` 尚未映射回面板名；
- 多实例面板再叠加 `event.instance` / `item_index` / `item_key`。

面板本身也可以写 `events`：等价于覆盖整个面板的一个自动区域，区域名用面板名。

### 2.4 clip

- `clip` 缺省 `true`；`draw` 和 `children` 都裁在面板矩形内。
- `clip: false` 仅用于确实需要溢出的特殊场景。
- 当前裁剪只作用于绘制，不改变布局；局部 `regions` 的命中尚未额外裁到面板矩形内。

## 3. 尺寸

`w` / `h` 和 `auto_size` 的优先级：

1. 显式 `w` / `h` 优先；
2. 该轴没有显式值时，`auto_size = "draw"` 才按 `draw` 包围盒测量；
3. 没有显式值且没有 `auto_size`：整份布局失败并报错，不静默塌成 0。

`auto_size` 目前只接受 `"draw"`。

### 3.1 draw 包围盒

用 `LayoutMeasureHook` 量 `text` 的宽高；其他元素用几何范围：

- `rect`：`x..x+w` / `y..y+h`；
- `line`：两端点 + `thickness / 2`；
- `circle`：`cx ± radius` / `cy ± radius`；
- `text`：`x` / `y` 为左上角，宽高由量尺给；
- `svg`：`x..x+size` / `y..y+size`；
- `img`：`x..x+w` / `y..y+h`；`auto_size` 下 `img` 必须显式写 `w/h`，否则报错；
- `path`：按命令的几何包围盒。

面板尺寸 = draw 包围盒覆盖的最小外框；padding 不额外加在 draw 外框上（draw 是外框坐标）。
没有量尺时退回字符估算。

## 4. 更新接口

`draw` 元素通过必需的 `name` 更新，name 比较大小写敏感。建议的 Rust 接口：

```rust
impl SelfDrawingPanel {
    /// 替换整份 draw list。
    fn set_draw(&mut self, draw: Vec<Element>);
    /// 按 name 替换一个 draw 元素。
    fn update_draw(&mut self, name: &str, element: Element) -> bool;
    /// 在指定下标插入一个 draw 元素。
    fn insert_draw(&mut self, index: usize, element: Element) -> bool;
    /// 删除一个 draw 元素。
    fn remove_draw(&mut self, name: &str) -> bool;
    /// 更新 / 删除一个局部区域。
    fn update_region(&mut self, name: &str, region: Region) -> bool;
    fn remove_region(&mut self, name: &str) -> bool;
}
```

单元素更新会重新走一次布局并重建 `Document`；`auto_size` 面板改 draw 内容后尺寸会跟着更新，
显式 `w/h` 不变。

## 5. 渲染模型

渲染器增加 `Element::SelfDrawingPanel`：

```text
SelfDrawingPanel {
  x, y, width, height,
  clip: bool,
  draw: Vec<Element>,     // 只含纯绘制元素
  children: Vec<Element>, // 动态控件
  regions: Vec<Region>,   // 面板局部命中区域，展开时按面板原点平移
}
```

- `draw_element` 在面板矩形处建立裁剪，先画 `draw`，再按原点偏移画 `children`，最后恢复裁剪。
- local regions 在 `Document.regions` 里展开成绝对区域；事件仍走现有 region → handler 链路。
- 自绘面板本身没有内置背景 / 边框；需要背景就用 draw 里的 `rect` 画。

## 6. 与页面树 / 窗口

- 页面树新增节点类型 `self_drawing_panel`，作为普通控件参与布局；`draw` 不占布局空间。
- `auto_size` 在页面树测量阶段调用 `LayoutMeasureHook`。
- 列表行、工具条、状态面板等都可以用自绘面板 + 区域实现；具体业务内容不进入库的设计。
- 窗口通过 `Document` 使用自绘面板；窗口与页面树的关系见 [`windows.md`](windows.md) 和
  [`ui-layout.md`](ui-layout.md)。

## 7. 实现顺序

1. `cloudui::element`：元素模型、draw / children 绘制、矩形裁剪、局部 regions 展开。
2. `cloudui::layout`：JSON / Rust 构造、`auto_size` 测量、页面树节点、单元素更新接口。
3. `cloudui::document`：事件上下文 `event.region`、更新后的 Document 替换。
4. 测试：固定尺寸 / auto_size、裁剪、region 命中、单元素更新、列表行示例。
5. 窗口集成。

## 8. 兼容性

- `panel` 的语义和行为不变。
- `draw` 里不允许放动态控件；要动态控件就放 `children`。
- 旧布局不受影响；不写 `self_drawing_panel` 就不会走新路径。
