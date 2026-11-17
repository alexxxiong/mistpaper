# 架构

```text
Windows / macOS / Linux 输入事件
              ↓
GPUI 原生窗口、文本系统和图形后端
              ↓
NSH GPUI Input：文本布局缓存、浮点滚动、选区和 IME
              ↓
NSH Rust 响应式组件（src/app.rs）
              ↓
Shelf：元数据 + Arc<str> 正文
              ↓
一个待保存快照 → 后台线程 → library.json.tmp → library.json
```

此项目取用原型中 GPUI 的渲染/输入层，并复用已经制作的 Rust 界面。实验中的 TSX + QuickJS + `nsh-gpui-host.exe` 组合没有进入发行目录。`nsh-platform-gpui` 的 `default-features = false` 禁用了其 JS、开发服务器和 WebView 默认入口；应用以 `AppSource::Rust` 直接启动。

滚动发生在原生 Input 内，不把位移、正文或文稿状态往返传给 JavaScript。静止文稿复用原生文本布局。`vendor/nsh-platform-gpui` 的应用补丁在构建字形列表前跳过屏幕外的行，并按可见文字分配绘制缓冲，不再按全文字形数量分配。输入、排版、选区、光标和 IME 逻辑保持上游路径。应用仍受上游 Input 的布局/绘制实现影响，原生编译本身不能保证任意文稿长度都达到固定帧率。

保存时只复制文稿元数据和正文 Arc 引用，待保存快照只有一个。后续输入会替换旧快照，不建立无限增长的全文保存队列。空闲 150 ms 后写入，持续输入则每秒发起保存。JSON 序列化、文件同步和替换由 `mistpaper-save` 线程执行。保存结果通过独立状态信号显示，不触发正文模型修改。

流动山景以 33 ms 定时更新玻璃 shader 的 phase；专注模式停止移动，点击暂停也会停止。保持原型中的 GPUI 填充式材质实现。它模拟应用内山景的透镜和模糊，不是对其他应用窗口或任意桌面背景的实时折射。

应用 ID 为 `studio.mistpaper.desktop`。新项目不会覆盖原型的 `studio.mistpaper.glass.*` 数据。可用 `MISTPAPER_DATA_DIR` 对每次性能测试隔离资料。

后续建议优先完善原生编辑能力、文稿导入导出和数据格式版本。更换文本组件、渲染依赖或保存策略前，先复测 30 万字中文稿、IME、选区、双指慢滑和惯性；保留同机同稿的比较证据。
