# 列表盒（list_box）

状态：设计初稿，待实现。

## 1. 定位

`list_box` 是 `SubpageContainer` 的一个具体控件类型，也叫“单列列表盒”。

- 只支持**单列、纵向**列表；
- 使用一个子页面模板重复生成列表项；
- 只使用 `subpage` 声明中的第一个模板，多余的只记警告；
- 固定高度，内容超过可视区域时出现内部垂直滚动条。

它不是一个通用表格，也不支持多列、多模板混排或横向滚动。

## 2. 类型关系

```text
Container（抽象）
├─ ControlsContainer（控件容器）
│  └─ Panel
└─ SubpageContainer（子页面容器）
   ├─ ListBox（list_box，单列列表盒）
   └─ 其他子页面容器（后续）
```

## 3. JSON 契约

```json
{
  "type": "list_box",
  "name": "list",
  "w": 320,
  "h": 240,
  "self_adaption": true,
  "align": "center",
  "gap": 0,
  "auto_item_gap_averaging": true,
  "the_gap_between_border_and_content": 2,
  "radius": 3,
  "fill": null,
  "edge_color": "#FFD2D2D2",
  "edge_width": 1,
  "allow_select": false,
  "auto_center_selected": false,
  "selected_index": -2,
  "subpage": ["row_template.json"],
  "repeat": "items",
  "key": "id",
  "item_height": "auto"
}
```

- `subpage` 是页面路径数组，只使用第一个元素，后面的元素忽略并记一条 `warn`。
- `repeat` / `key` / `item_height` 直接写在 `list_box` 顶层，不属于 `row_template.json`。
- `subpage[0]` 是子页面模板路径，按布局文件路径搜索链解析；顶层不再有独立的 `page` 字段。
- `repeat` 指向当前数据上下文里的数组；为空或缺失时只画背景，不画行。
- `key` 是稳定标识；数据更新时按 key 做 keyed diff。
- `item_height`：`"auto"`（缺省）或固定高度；`auto` 只决定高度，不决定宽度。
- `align`：`left` / `center` / `right`，行宽不铺满内容区时的水平对齐，缺省 `center`。
- `auto_item_gap_averaging`：绘制时是否把内容区剩余高度平均分到可见项目之间，缺省 `true`。
  只在内容高度超过内容区、需要滚动时生效；开启后可见行会以统一间隙铺满内容区。
  关闭后按 `gap` 定位，边缘剩余空间保留背景。
- `auto_center_selected`：选中项变化时是否垂直居中，缺省 `false`；只在 `allow_select = true`
  时有效。为 `false` 时只保证选中行完整可见。
- `the_gap_between_border_and_content`：边框内侧与内容区之间的间隙，缺省 2 物理像素；所有有边框控件通用。

## 4. 尺寸与测量

- `h` 始终是外框固定高度，必须有正值；不自动撑开。
- 内容区高度记作 `ch`：

```text
ch = h - edge_width * 2 - the_gap_between_border_and_content * 2
```

- 内容区宽度记作 `cw`；`self_adaption = false` 时：

```text
cw = w - edge_width * 2 - the_gap_between_border_and_content * 2 - (滚动条可见 ? 8 + 4 : 0)
```

- `edge_width` 是固定物理像素，不随 DPI 缩放；内容区计算按 `edge_width` 数值（点）扣减，
  默认 1 像素即 1 点；所有非 1 的边框也按同值点扣减，实际绘制仍按物理像素。
- `the_gap_between_border_and_content` 缺省 2 物理像素，是边框内侧到内容区的间隙；
  内容区计算固定按 2 点扣减，不受 DPI 缩放影响，实际绘制仍按物理像素。滚动条与内容之间的
  4 点间隙另算。该属性是所有有边框控件的通用属性，不只是 list_box。
- `self_adaption` 缺省 `true`：
  - `w` 逻辑无效；如果写了 `w`，忽略且不警告；
  - 先量子页面模板的自然宽度；没有项目时用模板宽度；
  - 若模板为空或宽度为 0，失败并进入延迟 panic；
  - 若内容高度超过 `ch`，滚动条出现，再从备选内容宽度中扣除滚动条宽度和 4 点间隙，
    重新量子页面内容；
  - 最多重新测量 3 次；
  - 3 次后如果行宽与滚动条状态仍有冲突，固定采用“保留滚动条 + 最后一次行宽”；
  - `item_height = "auto"` 时，行宽变化导致的换行和行高变化直接接受；
  - 如果滚动条出现后又因为缩窄导致行高变化、理论上又不需要滚动条，仍保留滚动条，避免抖动；
  - 最终控件外框宽度 = 最宽行内容宽度 + 滚动条占位（可见时）+ 边框 +
    `the_gap_between_border_and_content` 两侧。
- `self_adaption = false`：
  - `w` 是固定外框宽度；
  - 行按自然宽度测量，超过 `cw` 的部分裁剪，不因 `cw` 重新换行；
  - 行宽不强制铺满内容宽度；比内容宽的行按内容矩形裁剪；
  - 行宽小于内容宽度时，按 `align` 水平对齐，缺省居中。
- `gap` 是行与行之间的统一间距，缺省 0；不支持 `padding`。
- 行高由子页面根节点高度决定；`item_height` 固定时它是**最小高度**。
- 固定行高时，子页面更高就使用子页面的高度；子页面更矮则垂直居中。
- `item_height` 的固定值不包含 `gap`。
- 内容高度 = 所有行高 + `gap` × (行数 - 1)；只用于判断滚动条是否出现。
- 内容高度 `>` `ch` 时显示滚动条；等于时不显示。
- 绘制时只显示完整落在内容区内的行；滚动中也一样。
- `auto_item_gap_averaging = true` 时，把内容区剩余高度平均分到当前可见行之间，按统一间隙铺满内容区；
  关闭时按 `gap` 定位，内容区高度不是行高整数倍时边缘保留列表背景。单行高于内容区时无法完整显示，
  才允许部分可见。

## 5. 背景与区域

背景形状沿用 `panel` 语义：

| 属性 | 缺省 |
|---|---|
| `radius` | `3` |
| `fill` / `back_color` | 无填充 |
| `edge_color` | `#FFD2D2D2` |
| `edge_width` | `1` 物理像素，固定不随 DPI 缩放 |
| `the_gap_between_border_and_content` | `2`，边框内侧到内容区；所有有边框控件通用 |

- `fill` 和 `back_color` 等价；`edge_color` 是边框颜色，`edge_width` 是边框粗细。
- 边框绘制在控件边界之内；背景填充和内容区从边框内侧 +
  `the_gap_between_border_and_content` 开始。
- 背景是圆角矩形；内容**不按圆角裁剪**，按内容矩形裁剪。
- 区域就是整个背景外框；滚动条出现时，滚动条所在区域也属于 list_box 的命中范围，
  但滚动条自身优先命中。
- 点击空白背景或行内未覆盖区域派 `clicked`；点击滚动条和 4 点间隙不派。
- 点击行内控件时，`clicked` 优先，然后才是控件事件。

> 全局约定：所有控件的缺省边框颜色统一为 `#FFD2D2D2`，缺省边框粗细统一为 1 物理像素。

## 6. 滚动条

内部垂直滚动条沿用 `text_box` 的缺省参数：

- 宽 8 点；
- 与内容之间留 4 点间隙；
- 右侧内缩 2 点；
- 上下各内缩 `the_gap_between_border_and_content`，与内容区上下沿对齐；
- 轨道 `#FFC8C8C8`，滑块 `#FF888888`；
- 滑块最小长度为 1，具体长度按内容总高度和控件高度共同计算；
- 只有内容高度大于 `ch` 时显示；
- 不覆盖行内容；滚动条出现 / 消失时内容宽度变化，并触发一次重排和重绘。

交互：

- 拖动滑块滚动，拖动时实时更新，不走动画；拖动期间不跨控件捕获，指针离开后结束拖动；
- 绘制时滚动偏移吸附到行首；贴底时保留贴底偏移，保证最后一行完整；
- 开启 `auto_item_gap_averaging` 时，可见行的绘制、命中与行内编辑框位置统一使用平均后的间隙；
- 点击轨道空白（滑块上方或下方）直接把滑块定位到点击位置对应的滚动偏移（不按页翻），使用 140 ms 平滑动画；
- 滚轮按一行滚动，使用 140 ms 平滑动画；
- 滚轮在行内控件上时，先由子控件处理；子页面内没有控件支持滚轮时才交给列表；
- 键盘选中项变化后的自动滚动使用 140 ms 平滑动画；
- 滚动条本身不进入 `regions`，由 `list_box` 内部处理。

命中优先级见第 9 节。

## 7. 选择与键盘

- `allow_select` 缺省 `false`。
- `allow_select = false` 时：
  - `selected_index` 固定为 `-2`，表示不支持选择；
  - 强行设置 `selected_index` 直接 panic。
- `allow_select = true` 时：
  - 未写 `selected_index` 时缺省为 `-1`；
  - 初始值允许在加载时写成 `-1` / `0` 等合法值，且不派 `selected_index_changed`；
  - `selected_index` 从 0 开始；
  - `-1` 表示没有选择；
  - 支持方向键、PageUp、PageDown、Home、End；
  - PageUp / PageDown 按可视高度 `ch` 翻一页，选中项落在新页内与当前位置相对的位置；
  - 选中项变化后自动滚动到可见区域，使用 140 ms 平滑动画；`auto_center_selected = true`
    时改为垂直居中，缺省 `false`，且只在 `allow_select = true` 时有效；
  - 点击行会更新选中项；
  - `Home` / `End` 到第一个 / 最后一个逻辑项，也触发 `selected_index_changed`。
- 点击滚动条、背景空白不改变选中项。
- `list_box` 点击行时获得焦点；行内存在需要焦点的控件时，焦点优先给行内控件，
  方向键归该控件。
- 行内控件获得操作时，先派子页面控件自己的事件，再派
  `selected_item_operated`。

### 7.1 `item_state`

每个行实例的数据上下文增加只读状态：

| 值 | 含义 |
|---:|---|
| `0` | 正常态 |
| `1` | 选择态 |
| `2` | 高亮态（鼠标悬停） |

- 选择态吞并高亮态：同时悬停和选择时 `item_state = 1`。
- 选中项背景由子页面模板自己根据 `item_state` 绘制，`list_box` 不画选中背景。
- `focused` 与 `selected` 同义；行数据上下文同时提供 `selected` / `focused` 布尔变量，
  值等价于 `item_state == 1`。
- 行数据上下文提供 `hovered` 布尔变量，值等价于 `item_state == 2`。
- `allow_select = false` 时不会出现 `item_state = 1`。
- 鼠标悬停仍可产生 `item_state = 2`。
- 行模板按三种状态各展开一份元素；选择态优先，只有未选择的行才使用悬停态。
- 状态差异用展示字段的状态值表表达，例如：
  ```json
  "fill": { "state": { "0": "#FFFFFFFF", "1": "#FFE8F1FE", "2": "#FFF5F5F5" } }
  ```
  语法见 [`ui-layout.md`](ui-layout.md) 6.2.1。
- 三种状态共用同一份行高 / 行宽测量；状态值只改绘制，不参与布局。

## 8. 事件与 API

### 8.1 事件

| 事件 | 触发条件 |
|---|---|
| `selected_index_changed` | `allow_select = true` 且选中项变化 |
| `selected_item_operated` | `allow_select = true` 时，选中项内的控件事件触发之后 |
| `scroll_changed` | 滚动偏移、滚动条可见性变化 |
| `clicked` | 左键按下并在同一行 / 控件内抬起，且不是双击 |
| `double_clicked` | 左键在同一行 / 控件内完成双击；不再派 `clicked` |

事件顺序：

```text
clicked
→ selected_index_changed（如果选中项变化）
→ 行内控件事件
→ selected_item_operated（如果行内控件事件触发）
```

双击时：

```text
double_clicked（取代 clicked）
→ selected_index_changed（如果选中项变化）
→ 行内控件事件
→ selected_item_operated（如果行内控件事件触发）
```

- `selected_index_changed` 载荷包含 `index` 和 `key`；无选择时 `index = -1`。
- 点击滚动条、滚动条与内容之间的 4 点间隙：不触发 `clicked`。
- 点击已选中行的子控件：`clicked` 优先，然后控件事件，再 `selected_item_operated`；
  不触发 `selected_index_changed`。
- `selected_item_operated` 只统计操作类子事件：
  `click` / `mouse_scroll` / `key_pressed` / `text_changed` / `value_changed`；
  hover / leave / focused / unfocused 不算。
- `selected_item_operated` 载荷包含 `index`、`key`、子控件事件名、子控件 `control`；
  暂不带原始子事件载荷。
- `scroll_changed` 带滚动偏移（行）、内容高度、可视高度；按帧节流，停止时补发最终值。
- `clicked` 只响应鼠标左键；按下后必须在同一行 / 同一控件内抬起；
  拖动、滚动或离开控件后都不触发。
- 带 Ctrl / Alt / Shift 的左键点击也算 `clicked`。
- `clicked` 载荷包含点击位置、命中的行索引（空白区域为 `nil`）与修饰键
  `modifier_key`（字符串，如 `"ctrl+alt"`；无修饰键为空字符串）。
- `clicked` 不是可取消事件：处理器返回后，后续的选中变化、行内控件事件和
  `selected_item_operated` 仍按顺序继续；只有窗口已被销毁时才停止。
- 双击时只派一次 `double_clicked`，不派 `clicked`；其载荷与 `clicked` 相同，
  包含位置、行索引和 `modifier_key`。
- 双击也会按普通行点击的规则更新 `selected_index`；如果选中项因此变化，先派
  `double_clicked`，再派 `selected_index_changed`。
- 双击的行内控件事件是否按普通点击处理，沿用子控件自己的双击规则；`list_box`
  不额外吞掉子控件事件。

### 8.2 API

至少提供：

- `scroll_to(index)`：把指定项目滚到控件垂直居中位置；索引越界 panic。
  - 如果目标行及之前行的实际高度还没测过，先测量目标行及其前面的所有行，再计算居中偏移。
- `scroll_offset`：只读整数，当前第一个可见行的行偏移，单位是行（项目）。
  平滑滚动停在两行之间时，报告第一个可见行（向下取整）。
- `item_count`：只读逻辑项目数；方法 `get_item_count()` 返回同一值。
- `get_item(index)`：返回指定项目的子页面实例句柄；索引越界 panic。
  - 句柄至少提供 `index` / `key` / `instance` / `control_path(name)` / `get_text(name)` /
    `set_text(name, text)` / `focus(name)`。
  - 虚拟化时目标行不在可见窗口，按需实例化该行并返回句柄；不改变滚动位置和选中项。
  - 按需实例化的行一律计入该 list_box 的 `max_nodes`；
  - 行滚出可见区域后仍保留，直到列表数据更新或显式释放；
  - 同一 index 多次调用返回同一个句柄；
  - 回收后旧句柄失效，再调用 `get_item` 重新创建。
- `selected_index`：可读可写；写入时：
  - 值没变化不派 `selected_index_changed`；
  - 合法值变化才派 `selected_index_changed`；
  - 按 `auto_center_selected` 滚动：`false` 只保证目标行可见，`true` 垂直居中；
  - 越界直接 panic。
- `allow_select = false` 时读取 `selected_index` 为 `-2`，写入直接 panic。

数据更新使用 keyed diff：

- 保留滚动位置；
- 保留焦点；
- `allow_select = true` 时按 key 保留选中项；
- 选中项 key 还在但位置改变：更新 `selected_index`，并派 `selected_index_changed`；
- 选中项 key 被删除：派 `selected_index_changed`；
  - 如果还有相邻项，优先选择上一个；没有上一个才选择下一个；
  - 如果已经没有项目，`selected_index = -1`；
- 选中项 key 不变但内容更新：保持选中和滚动位置，不派选中变化事件。

## 9. 命中、渲染与裁剪

### 9.1 命中优先级

```text
1. 滚动条轨道（可见时）
2. 项目行 / 行内控件
3. list_box 背景区域
```

- 滚动条命中范围是轨道矩形；4 点间隙不归滚动条，但也不触发 `clicked`。
- 指针在滚动条上时，行不进入 hover，不触发行点击或选中变化。
- 拖动滑块期间不跨控件捕获；指针离开轨道后结束拖动，不把后续事件交给行。
- 滚动条隐藏时没有滚动条命中层。
- 滚动条交互不改变 `selected_index`。
- 滚轮在行内控件上时，子控件优先；子页面内没有控件支持滚轮时才滚动列表。
- 滚轮在滚动条上时，由列表滚动。

### 9.2 渲染顺序

```text
背景填充
→ 滚动条
→ 项目内容
→ 边框（最后再描一次）
```

- 边框最后绘制，避免填充覆盖边框。
- 项目内容按内容矩形裁剪，不按圆角裁剪。
- 滚动条在边框以内，不覆盖行内容。
- 滚动条显示 / 隐藏后重新布局并重绘。

## 10. 生命周期与性能

- 数据更新使用 keyed diff，只更新对应项目，复用子页面实例。
- 支持虚拟化：只实例化可见行 + overscan；滚轮和滚动条滚动都只重建可见窗口。
  - 固定 `item_height` 直接按行高计算；`auto` 行高维护逐行高度缓存，未测量行先按估算高度定位，
    进入可见窗口后再测量并修正总高度。
- 逻辑项目上限 `65535`。
- 实际实例化节点上限 `100000`（按每个 list_box 计算；虚拟化时统计可见实例、overscan，
  以及 `get_item` 按需实例化并保留的行）。
- 单个子页面模板节点上限 `512`（只统计实际控件节点，不统计 `draw` 绘制元素）。
- 子页面嵌套深度上限 `1`：子页面内部不能再出现 `subpage` 或 `list_box`。
- 模板文件大小沿用 4 MB 上限。
- 以上限制在加载 / 校验阶段检查；错误先全部收集，不在滚动、渲染、事件处理阶段立即 panic，
  等 UI 进程创建完成后统一触发恐慌。
- `scroll_to` / `selected_index` 等运行时参数错误直接 panic。

## 11. 校验与失败

- `h` 缺失或不为正：失败。
- `self_adaption = false` 时 `w` 缺失或不为正：失败。
- `subpage` 不是字符串数组或为空：失败。
- 多个 `subpage` 路径：只使用第一个，其余记 `warn`。
- 模板缺失、必要字段缺失、重复 key：失败；按第 10 节延迟 panic。
- 行高测得为 0：先取当前子页面高度，仍为 0 才失败。
- `allow_select = false` 时设置 `selected_index`：运行时直接 panic；加载时显式写成非 `-2` 收集为校验错误。
- `scroll_to(index)` / `get_item(index)` 索引越界：运行时直接 panic。
- `selected_index` 在 `allow_select = true` 时超出 `-1..count-1`：运行时直接 panic。
- 项目数超过 65535、实例化节点超过 100000：失败。
- 子页面内部再嵌套子页面 / `list_box`：失败。
