# 侧边栏宽度调整后终端内容区域未 resize 修复计划

## 📋 问题概述

**现象**：用户拖拽调整侧边栏宽度后，右侧终端内容区域没有随之调整尺寸。

**影响**：终端内容区域保持原有尺寸，导致显示不正确，用户体验差。

---

## 🔍 根本原因分析

经过详细代码分析，发现存在两个独立但相关的问题：

### 问题 1：使用了错误的函数计算侧边栏宽度

**位置**：`wezterm-gui/src/termwindow/resize.rs` 第 210 行和第 255 行

**问题**：
```rust
let vertical_tab_bar_width =
    Self::tab_bar_pixel_width_impl(&self.config, self.show_tab_bar) as usize;
```

`tab_bar_pixel_width_impl` 是静态方法，只读取配置文件的固定值 `config.tab_bar_vertical_width`，**完全忽略**用户拖拽后设置的运行时覆盖值 `vertical_tab_bar_width_override`。

**正确的函数**：
```rust
pub fn tab_bar_pixel_width(&self) -> f32 {
    if self.config.tab_bar_vertical && self.show_tab_bar {
        self.vertical_tab_bar_width_override  // ✅ 优先使用运行时覆盖值
            .unwrap_or(self.config.tab_bar_vertical_width as f32)
    } else {
        0.
    }
}
```

### 问题 2：拖拽后没有触发 resize

**位置**：`wezterm-gui/src/termwindow/mouseevent.rs` 第 425-459 行

**问题**：
```rust
fn drag_vertical_tab_bar_resize(...) {
    // ...
    self.vertical_tab_bar_width_override = Some(new_width);  // ✅ 更新覆盖值
    self.invalidate_fancy_tab_bar();  // ❌ 只重绘标签栏
    context.invalidate();  // ❌ 只触发渲染，没有 resize
}
```

拖拽过程中只更新了 `vertical_tab_bar_width_override` 并触发重绘，**没有调用 `apply_dimensions` 重新计算终端尺寸**。

---

## ✅ 修复方案

### 方案选择：拖拽结束时触发 resize（方案 B）

**优点**：
- 性能更好，只在拖拽结束时计算一次
- 实现简单，逻辑清晰

**缺点**：
- 拖拽过程中终端内容不会实时调整（但这是可接受的）

---

## 🔧 详细修复步骤

### 步骤 1：修改 resize.rs（必须）

**文件**：`wezterm-gui/src/termwindow/resize.rs`

#### 修改点 1.1：第 209-210 行

**修改前**：
```rust
let vertical_tab_bar_width =
    Self::tab_bar_pixel_width_impl(&self.config, self.show_tab_bar) as usize;
```

**修改后**：
```rust
let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;
```

**上下文**（用于定位）：
```rust
// 第 205-214 行
let pixel_height = (rows * self.render_metrics.cell_size.height as usize)
    + (padding_top + padding_bottom)
    + (border.top + border.bottom).get() as usize
    + tab_bar_height as usize;

let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;  // ← 修改这里
let pixel_width = (cols * self.render_metrics.cell_size.width as usize)
    + (padding_left + padding_right)
    + (border.left + border.right).get() as usize
    + vertical_tab_bar_width;
```

#### 修改点 1.2：第 254-255 行

**修改前**：
```rust
let vertical_tab_bar_width =
    Self::tab_bar_pixel_width_impl(&self.config, self.show_tab_bar) as usize;
```

**修改后**：
```rust
let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;
```

**上下文**（用于定位）：
```rust
// 第 248-262 行
let padding_left = config.window_padding.left.evaluate_as_pixels(h_context) as usize;
let padding_top = config.window_padding.top.evaluate_as_pixels(v_context) as usize;
let padding_bottom =
    config.window_padding.bottom.evaluate_as_pixels(v_context) as usize;
let padding_right = effective_right_padding(&config, h_context);

let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;  // ← 修改这里
let avail_width = dimensions
    .pixel_width
    .saturating_sub(
        (padding_left + padding_right) as usize
            + (border.left + border.right).get() as usize,
    )
    .saturating_sub(vertical_tab_bar_width);
```

---

### 步骤 2：修改 mouseevent.rs（必须）

**文件**：`wezterm-gui/src/termwindow/mouseevent.rs`

#### 修改点 2.1：第 128-139 行的 Release 事件处理

**修改前**：
```rust
WMEK::Release(ref press) => {
    self.current_mouse_capture = None;
    self.current_mouse_buttons.retain(|p| p != press);
    if press == &MousePress::Left && self.window_drag_position.take().is_some() {
        // Completed a window drag
        return;
    }
    if press == &MousePress::Left && self.dragging.take().is_some() {
        // Completed a drag
        return;
    }
}
```

**修改后**：
```rust
WMEK::Release(ref press) => {
    self.current_mouse_capture = None;
    self.current_mouse_buttons.retain(|p| p != press);
    if press == &MousePress::Left && self.window_drag_position.take().is_some() {
        // Completed a window drag
        return;
    }
    if press == &MousePress::Left {
        if let Some((item, _)) = self.dragging.take() {
            // 如果是侧边栏拖拽，触发 resize 重新计算终端尺寸
            if item.item_type == UIItemType::VerticalTabBarResize {
                if let Some(window) = self.window.as_ref() {
                    self.apply_dimensions(&self.dimensions, None, window);
                }
            }
            return;
        }
    }
}
```

**说明**：
- 在鼠标左键释放时，检查是否是侧边栏拖拽（`UIItemType::VerticalTabBarResize`）
- 如果是，调用 `apply_dimensions` 重新计算终端尺寸
- 使用 `self.window.as_ref()` 获取 Window 对象

---

## 📝 完整代码修改示例

### 文件 1：resize.rs

```rust
// 第 209-210 行
// 修改前：
let vertical_tab_bar_width =
    Self::tab_bar_pixel_width_impl(&self.config, self.show_tab_bar) as usize;

// 修改后：
let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;


// 第 254-255 行
// 修改前：
let vertical_tab_bar_width =
    Self::tab_bar_pixel_width_impl(&self.config, self.show_tab_bar) as usize;

// 修改后：
let vertical_tab_bar_width = self.tab_bar_pixel_width() as usize;
```

### 文件 2：mouseevent.rs

```rust
// 第 128-139 行
// 修改前：
WMEK::Release(ref press) => {
    self.current_mouse_capture = None;
    self.current_mouse_buttons.retain(|p| p != press);
    if press == &MousePress::Left && self.window_drag_position.take().is_some() {
        // Completed a window drag
        return;
    }
    if press == &MousePress::Left && self.dragging.take().is_some() {
        // Completed a drag
        return;
    }
}

// 修改后：
WMEK::Release(ref press) => {
    self.current_mouse_capture = None;
    self.current_mouse_buttons.retain(|p| p != press);
    if press == &MousePress::Left && self.window_drag_position.take().is_some() {
        // Completed a window drag
        return;
    }
    if press == &MousePress::Left {
        if let Some((item, _)) = self.dragging.take() {
            // 如果是侧边栏拖拽，触发 resize 重新计算终端尺寸
            if item.item_type == UIItemType::VerticalTabBarResize {
                if let Some(window) = self.window.as_ref() {
                    self.apply_dimensions(&self.dimensions, None, window);
                }
            }
            return;
        }
    }
}
```

---

## 🧪 测试验证步骤

### 1. 编译测试

```bash
cd D:\work\wezteam
cargo build
```

确保编译通过，没有语法错误。

### 2. 功能测试

#### 测试用例 1：基本拖拽测试
1. 启动 WezTerm
2. 启用垂直标签栏（如果未启用）
3. 拖拽侧边栏边缘，调整宽度
4. **预期结果**：释放鼠标后，终端内容区域应该立即调整到新的尺寸

#### 测试用例 2：最小宽度测试
1. 尝试将侧边栏拖拽到最小宽度（80px）
2. **预期结果**：侧边栏不应该小于 80px，终端内容区域应该正确调整

#### 测试用例 3：最大宽度测试
1. 尝试将侧边栏拖拽到最大宽度（窗口宽度的 50%）
2. **预期结果**：侧边栏不应该超过窗口宽度的 50%，终端内容区域应该正确调整

#### 测试用例 4：右侧标签栏测试
1. 配置标签栏显示在右侧
2. 拖拽调整宽度
3. **预期结果**：终端内容区域应该正确调整

#### 测试用例 5：窗口 resize 测试
1. 拖拽调整侧边栏宽度
2. 然后调整窗口大小
3. **预期结果**：侧边栏宽度应该保持，终端内容区域应该正确计算

### 3. 性能测试

1. 快速连续拖拽侧边栏
2. **预期结果**：应该流畅，不应该有卡顿或延迟

### 4. 边界测试

1. 在不同 DPI 显示器上测试
2. 在不同字体大小下测试
3. 在不同窗口大小下测试

---

## ⚠️ 注意事项

### 1. 必须同时修复两个问题

**重要**：只修复其中一个问题无法解决根本问题！

- 只修复问题 1：拖拽后仍然不会触发 resize
- 只修复问题 2：即使触发了 resize，计算出的宽度也是错的

### 2. 代码审查要点

- 确认修改的是实例方法 `self.tab_bar_pixel_width()`，不是静态方法
- 确认在 Release 事件中正确判断了 `UIItemType::VerticalTabBarResize`
- 确认使用了 `self.window.as_ref()` 而不是其他方式获取 Window

### 3. 潜在风险

- **性能影响**：拖拽结束时调用 `apply_dimensions` 会重新计算终端尺寸，但影响应该很小
- **兼容性**：确保修改不影响其他拖拽操作（分割线拖拽、滚动条拖拽等）

### 4. 回滚方案

如果修改导致问题，可以快速回滚：

```bash
git diff  # 查看修改
git checkout -- wezterm-gui/src/termwindow/resize.rs
git checkout -- wezterm-gui/src/termwindow/mouseevent.rs
```

---

## 📊 修改影响范围

| 文件 | 修改行数 | 影响范围 | 风险等级 |
|------|---------|---------|---------|
| resize.rs | 2 处（第 210、255 行） | 终端尺寸计算 | 低 |
| mouseevent.rs | 1 处（第 128-139 行） | 鼠标事件处理 | 低 |

**总体风险**：低。修改点明确，影响范围小，逻辑简单。

---

## 🎯 验收标准

修复完成后，应该满足以下标准：

- ✅ 拖拽侧边栏后，终端内容区域正确调整
- ✅ 终端的行列数正确计算
- ✅ 性能流畅，无卡顿
- ✅ 边界情况正确处理（最小/最大宽度）
- ✅ 不影响其他功能（分割线拖拽、窗口 resize 等）
- ✅ 编译通过，无警告
- ✅ 所有测试用例通过

---

## 📚 相关代码位置

### 关键文件

1. **resize.rs**：终端尺寸计算核心逻辑
   - `apply_dimensions()`：应用新尺寸，计算行列数
   - `resize()`：窗口 resize 事件处理

2. **mouseevent.rs**：鼠标事件处理
   - `drag_vertical_tab_bar_resize()`：侧边栏拖拽处理
   - `WMEK::Release`：鼠标释放事件

3. **tab_bar.rs**：标签栏相关
   - `tab_bar_pixel_width()`：实例方法，读取运行时覆盖值
   - `tab_bar_pixel_width_impl()`：静态方法，只读取配置值

### 关键数据结构

- `vertical_tab_bar_width_override`：运行时侧边栏宽度覆盖值
- `dimensions`：窗口尺寸
- `terminal_size`：终端尺寸（行列数）

---

## 💡 扩展优化建议（可选）

如果后续需要进一步优化，可以考虑：

### 优化 1：拖拽过程中实时 resize

在 `drag_vertical_tab_bar_resize` 中实时触发 resize：

```rust
fn drag_vertical_tab_bar_resize(...) {
    // ...
    let width_changed = self.vertical_tab_bar_width_override != Some(new_width);
    self.vertical_tab_bar_width_override = Some(new_width);
    
    if width_changed {
        if let Some(window) = self.window.as_ref() {
            self.apply_dimensions(&self.dimensions, None, window);
        }
    }
    // ...
}
```

**优点**：用户体验更好，所见即所得
**缺点**：性能开销稍大

### 优化 2：添加防抖

限制 resize 触发频率，避免快速拖拽时频繁计算：

```rust
use std::time::Instant;

// 在 TermWindow 中添加
last_tab_bar_resize_time: Option<Instant>,

// 在拖拽时判断
const RESIZE_DEBOUNCE_MS: u64 = 50;
if let Some(last) = self.last_tab_bar_resize_time {
    if last.elapsed().as_millis() < RESIZE_DEBOUNCE_MS as u128 {
        return;  // 跳过，等待下次
    }
}
self.last_tab_bar_resize_time = Some(Instant::now());
```

---

## 📞 联系方式

如有问题，请联系：
- 问题分析者：AI Coding Assistant
- 文档创建时间：2026-07-02
- 相关 Issue：侧边栏宽度调整后终端内容区域未 resize

---

## ✅ 完成检查清单

修复完成后，请确认：

- [ ] 已修改 resize.rs 第 210 行
- [ ] 已修改 resize.rs 第 255 行
- [ ] 已修改 mouseevent.rs Release 事件处理
- [ ] 编译通过
- [ ] 基本拖拽测试通过
- [ ] 最小宽度测试通过
- [ ] 最大宽度测试通过
- [ ] 右侧标签栏测试通过
- [ ] 窗口 resize 测试通过
- [ ] 性能测试通过
- [ ] 代码已提交

---

**祝开发顺利！**
