# 自绘渲染器

## 1. 目标

`cloudui` 的窗口内容全部由库自己画：布局引擎给出绝对几何，渲染器把一帧画成位图，窗口后端只负责贴图和
把输入坐标传回来。

```text
PageTree -> Layout -> Document -> Renderer -> Rendered（位图 + 命中区域）
                                      |
                                   Window Backend
```

- 渲染器不依赖具体窗口系统；Windows 分层窗口只是其中一种后端。
- 渲染结果必须能逐像素复现，方便做快照测试和主题预览。
- 一帧的产物包括位图和命中区域；输入命中不重新排版，直接查区域 / 元素几何。

## 2. 输出契约

```rust
pub struct Document {
    pub width: f32,
    pub height: f32,
    pub elements: Vec<Element>,
    pub regions: Vec<Region>,
    pub text_hosts: Vec<TextHost>,
}

pub struct Rendered {
    /// 预乘 RGBA 位图。
    pub bitmap: Bitmap,
    /// 绝对坐标的命中区域。
    pub regions: Vec<Region>,
    /// 元素的绝对命中框，供调试与回退命中。
    pub element_rects: Vec<Rect>,
}

pub trait Renderer {
    fn render_document(&mut self, document: &Document) -> Result<Rendered, RenderError>;
}
```

- 位图统一用**预乘 RGBA**；Windows 后端在贴图前转成 BGRA。
- 尺寸单位是逻辑点；后端按 DPI / 缩放换算成物理像素。
- 渲染失败返回错误，窗口保持上一帧，不显示半成品。

## 3. 技术选型

| 层 | 选择 | 理由 |
|---|---|---|
| 2D 栅格 | `tiny-skia` | 纯 Rust、CPU、依赖少；面板 / 弹窗尺寸下延迟足够低 |
| 文字 | `cosmic-text` | fontdb 找字体、rustybuzz 整形、swash 栅格，带回退链与彩色 emoji |
| SVG | `usvg` + `resvg` | 矢量图标按目标尺寸光栅化，可按 alpha 染色 |
| 图片 | `image` | PNG 等常见格式解码后按目标矩形缩放 |
| 主题 | JSON | 结构简单，宿主和工具都容易读写 |

不上 GPU：`cloudui` 面向的是中小尺寸的浮层和面板，CPU 栅格更简单、结果更稳定。将来如果要画大面积动画，
再评估 GPU 后端，但不改变 `Document -> Rendered` 的契约。

## 4. 元素

| 元素 | 是什么 |
|---|---|
| `text` | 文本，带字体槽、字号、颜色、行距 |
| `svg` | SVG 图形，可按 alpha 染色 |
| `img` | 图像，按目标矩形缩放 |
| `line` | 线条（水平 / 垂直 / 任意角度） |
| `rect` | 矩形，含圆角矩形与圆形 |
| `path` | 路径（直线 / 三次贝塞尔 / 闭合，非零环绕填充） |

- 元素按 `Document.elements` 的顺序绘制；后写的盖住先写的。
- 每个元素可以有裁剪矩形；`self_drawing_panel` 用它把 draw / children 裁在面板内。
- 元素的几何就是它的绘制范围；命中优先使用显式 `Region`，没有区域时回退到元素命中框。
- 阴影不是单独的元素：它由圆角矩形、模糊和颜色合成，作为 `rect` 的扩展属性。

## 5. 文本

- 字体按字族名按需加载，不扫描全系统字体；系统字体作为回退链。
- 整形必须带 locale，避免 Han 统一编码下选错中日字形。
- 支持彩色 emoji（COLR / sbix 等由 swash 处理）。
- 文本测量与绘制共用同一套字形管线：`LayoutMeasureHook` 量出的宽高必须和最终位图一致。
- 多行高度 = 行高 × 行数 + 行距 × (行数 − 1)。
- 字体族、字号、行距由控件属性 / 主题提供；渲染器不猜测业务语义。

## 6. 窗口后端

Windows 后端使用分层窗口：

- 窗口类不处理 `WM_PAINT`；没有「系统擦掉画布」的时机；
- 渲染器输出预乘 RGBA，后端转 BGRA 后 `UpdateLayeredWindow` 整块贴上；
- 阴影由渲染器画，分层窗口没有系统阴影；
- 所有尺寸按 DPI / 96 换算；换显示器或缩放变化时重新出图；
- 不抢系统焦点：窗口使用不激活样式，鼠标穿透由窗口模板的焦点状态控制。

其它平台可以实现同一份 `Rendered` 契约（例如软件窗口 / 合成器纹理），上层布局和绘制代码不变。

## 7. 更新与缓存

- 一帧对应一份 `Document`；内容变了才重新渲染并贴图，没变就不贴。
- 整块位图重画是缺省策略；在性能不够时再加局部脏矩形或分层缓存，但对外契约不变。
- 静态背景、图标、文字可以按元素缓存；动画（高亮滑动、展开收起）通过生成中间帧实现。
- 渲染器可以在内部缓存字形、SVG 光栅结果和图片解码结果；缓存键必须包含尺寸、DPI 和颜色变换。
- 窗口后端按「尺寸 / DPI / 帧序号」判重，避免重复贴同一张图。

## 8. 主题

渲染参数由 `Theme` 提供：

- 颜色：背景、文字、边框、强调色、阴影、状态色；
- 字体：字族、字号、行距、字重；
- 形状：圆角、间距、留白、阴影大小与模糊；
- 主题只影响绘制，不改变页面树结构和布局规则。

主题文件与加载接口见 [`theme.md`](theme.md)。

## 9. 非目标

- 不做原生控件 toolkit：输入框、表格、无障碍等复杂控件不在自绘范围内；需要时由宿主或平台控件提供。
- 不引入 WebView / Flutter 作为渲染层；`cloudui` 的契约是 Rust 数据结构和位图。
- 不把业务逻辑放进渲染器：渲染器只认识 `Document` / `Element` / `Region` / `Theme`。

## 10. 相关文档

- 窗口模型：[`windows.md`](windows.md)
- 自绘面板：[`self-drawing-panel.md`](self-drawing-panel.md)
- 页面树与布局：[`ui-layout.md`](ui-layout.md)
- 主题：[`theme.md`](theme.md)
