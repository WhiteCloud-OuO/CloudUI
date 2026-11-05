# 内置对话框（dialog）

## 0. 定位

`dialog` 是**辅助窗口模板的一种内置种类**（`AssetKind::Dialog`），正文布局由库内部代码生成，
用户不需要写 JSON，也不负责摆放标题栏 / 内容区 / 按钮组。

当前提供两种具体对话框：

- **消息对话框**（`MessageDialog`）：标题栏 + 图标 + 正文 + 1–4 个按钮，返回按钮下标；
- **文本输入对话框**（`TextInputDialog`）：标题栏 + 说明文字 + 单行 / 多行编辑框 + 确定 / 取消，返回字符串。

它们共用：

- `dialog::layout`：宽度、留白、正文折行、标题文字、按钮绘制；
- `dialog::DialogStyle`：字体、字号与颜色；
- `dialog::BlockingPanel`：同步模态宿主（禁用所有者、屏蔽编辑框、嵌套消息循环、恢复）；
- `AuxiliaryWindowTemplate::dialog`：owned window、置顶、自动模态、拖拽。

## 1. 类与属性

**所有对话框都是类**：先 `new` 建实例，再改属性，最后 `show(self)`。

```rust
let mut dialog = MessageDialog::new(title, text, icon, &["确定", "取消"]);
dialog.style.text_size = Some(15.0);
let index = dialog.show();           // u8：被点按钮下标 0..=3

let mut input = TextInputDialog::new(title, text, allow_multiline, default_text);
input.style.title_color = Some(color);
let text = input.show();             // String
```

- `show(self)` **吃掉实例**：Rust 借编译器保证 `show` 之后不能再改属性。
- 实例上的内容字段（`title` / `text` / `icon` / `buttons` / `allow_multiline` / `default_text`）
  和 `style` 都是公开的，`show` 之前随便改。

### 1.1 `DialogStyle`

| 属性 | 缺省 | 意思 |
|---|---|---|
| `font` | 渲染器缺省槽 | 正文 / 标题 / 按钮用的字体槽 |
| `text_size` | 14 | 正文与按钮字号（点） |
| `title_size` | 标题栏缺省（13） | 标题字号（点） |
| `background` | 主题背景色 | 面板底色 |
| `title_color` | 主题次要文字色 | 标题颜色 |
| `text_color` | 主题正文色 | 正文 / 编辑框文字颜色 |
| `button_background` | 主题强调色 | 按钮底色 |
| `button_text_color` | 主题强调文字色 | 按钮文字颜色 |

颜色覆盖写进主题副本；字号 / 字体槽作为参数传给标题、正文、按钮与编辑框的排版。
未设置的项跟随当前窗口的深 / 浅主题。

### 1.2 共用尺寸

- 宽度固定 380 点；内容区左右与底部留白 20 点；
- 标题栏与主窗口一致：高 36 点、左留白 16 点、13 点字号、次要文字色；
  **没有关闭按钮，标题栏下不画分隔线**；
- 正文行距 4 点；按钮高 32 点、间距 8 点、最小宽 72 点；
- 从窗口边缘越出去的内容不显示（整块绘制按文档尺寸裁剪）。

## 2. 同步模态宿主

`show` 是**同步阻塞**的：

1. 取线程本地「当前窗口」（`crate::current`）作为所有者、渲染器与主题来源；没有当前窗口时异常终止；
2. 生成 `Document`，用 `AuxiliaryWindowTemplate::dialog` 建 owned window；
3. 禁用所有者窗口（`EnableWindow(FALSE)`）；
4. 挡住所有者名下的 RichEdit 宿主：它们是**独立顶层窗口**，`EnableWindow` 管不到，
   需要从当前窗口上下文拿到宿主列表，逐个 `set_input_blocked(true)`（鼠标穿透 + 放掉键盘焦点）；
5. 记录当前焦点，居中显示对话框；
6. 跑嵌套 `GetMessageW` 循环，直到按钮回调设置结果并 post 唤醒消息，或窗口被销毁；
7. 恢复：取消宿主屏蔽、重新启用所有者、把焦点还给打开前的窗口。

`buttons` 为空或超过 4 个、没有当前窗口、渲染器未就绪等非法状态一律
`std::process::abort()`，不使用 panic。

> 所有者窗口在 `Application::update` 里打开对话框时，正处在自己的绘制 / 更新借用中；
> 因此宿主列表必须由窗口在进入 UI 上下文时携带，不能在对话框里回查窗口对象。

## 3. 消息对话框

```rust
MessageDialog::new(title, text, icon, buttons: &[&str]) -> MessageDialog
```

- **布局**：标题栏 / 内容区（左 SVG + 右 `TextBlock`）/ 按钮组；
- **文本**：`\n` 硬换行，按内容区宽度自动折行；`\\` 表示反斜杠；
- **图标**：`""`（不画图）/ `failure` / `success` / `query` / `warning`；
  四张 SVG 直接作为字符串常量内嵌，保留源色，不跟随主题染色；
- **按钮**：1–4 个文字按钮，右对齐；返回被点按钮下标 `0..=3`；
- **没有关闭按钮**，必须点一个按钮；窗口被外部销毁时按 0 处理。

## 4. 文本输入对话框

```rust
TextInputDialog::new(title, text, allow_multiline, default_text) -> TextInputDialog
```

- **布局**：标题栏 / 说明文字 + 编辑框 / 确定 + 取消；
- `allow_multiline = true` 用多行 `text_box`（高 110 点），否则用单行 `text_edit`（高 34 点）；
- 背景与边框由 `Element::TextEdit` / `Element::TextBox` 画，真正的输入由 `TextHostWindow`
  盖在元素的 `edit_rect()` 上（避开边框与圆角）；
- 打开即把焦点交给编辑框并**全选缺省文本**，直接输入就是替换；
- **确定** → 编辑框当前内容；
- **取消** → `default_text`；`default_text` 为空时返回空串；
- 单行模式下，`default_text` 里的真实换行显示为字面量 `\n`，确定后原样返回。

## 5. 拖拽

对话框面板可以拖动：在非按钮区域按下左键，超过 4 像素后开始拖拽；`SetCapture` 保证拖出窗口仍连续。

- 对话框和**所有者**按相同位移一起移动；
- 所有者仍是模态状态：`EnableWindow` 保持禁用，拖拽只改窗口位置，不恢复鼠标 / 键盘输入；
- 拖拽后的左键抬起**不再派发点击**，避免从空白处拖到按钮上误触；
- `WM_CAPTURECHANGED` 清掉旧起点。

### 5.1 跟随着一起移动的窗口

编辑框宿主与窗口蒙版都是独立顶层窗口，不会跟着父窗口或所有者移动。对话框拖拽时按相同位移搬运：

- 所有者名下的 RichEdit 宿主（通过当前窗口上下文传入，`TextHostWindow::move_by` 保持 bounds 缓存一致）；
- 所有者当前的窗口蒙版；
- 文本输入对话框自己的编辑框宿主。

宿主窗口不能在下一帧再按窗口位置重算一次，否则会**二次位移**；`move_by` 同时更新宿主的 bounds 缓存，
后续 `sync_text_edits` 看到位置已一致就跳过。

## 6. 光标

- 辅助窗口类注册箭头光标，窗口过程在 `WM_SETCURSOR` / `HTCLIENT` 时显式恢复箭头；
- 编辑框宿主的 I 型光标由 `text_edit` 自己设置；
- 鼠标从编辑框移回对话框面板时，光标恢复箭头，不会停在 I 型。

## 7. 生命周期与模态栈

- 创建即模态：`Assets::add` 看到 `AssetKind::Dialog` 就压到模态栈顶，调用方不需要、也不允许再
  `set_modal(true)`；
- 栈顶原来有普通模态（或外层 dialog）时先临时挂起，dialog 销毁后恢复下一个；嵌套 dialog 同理；
- dialog 位于最顶层，并且是所有者的 owned window；每次显示 / 开蒙版后再 `HWND_TOP`；
- 创建时如果焦点在别的资产上，先经桥接交还所有者，再由所有者指定给 dialog；
- 生命周期内 `set_modal(false)` 与 `hide` 都不允许；非法调用直接异常终止；
- 可选蒙版：`DialogOptions::mask = true` 时 dialog 记录所有者原来的蒙版并切到半透明遮罩，
  销毁前恢复原蒙版。
