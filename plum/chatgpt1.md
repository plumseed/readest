请完整分析当前 Readest 项目中“阅读字体”的实现方式，但暂时不要修改任何代码。

目标是后续加入一个“系统字体”选项，让 Android 版阅读正文时使用手机当前系统默认字体，而不是必须导入字体文件。

请重点调查：

1. 字体设置页面位于哪些文件；
2. 内置字体、自定义字体和 EPUB 自带字体分别如何管理；
3. 用户选择的字体名称保存在哪里，使用什么数据结构；
4. 阅读正文最终由什么组件渲染：
   - Android 原生 TextView / Compose
   - WebView
   - iframe
   - foliate-js
   - CSS 注入
   - 其他方案
5. font-family 最终在什么位置被应用到书籍正文；
6. Android、Windows、macOS、Linux 和 Web 是否共用同一套字体逻辑；
7. Android 端是否已有 Tauri、Kotlin、Java 或 Capacitor 原生桥接代码；
8. 当前代码是否已经使用 system-ui、sans-serif、serif、ui-sans-serif 或系统字体回退；
9. EPUB 内嵌 CSS 或内嵌字体是否会覆盖用户选择的字体；
10. 实现“跟随 Android 系统字体”最安全、最小改动的方案是什么。

请特别区分下面两种需求：

A. 使用 Android 默认 UI 字体，例如通过 CSS 的 system-ui 或 sans-serif；
B. 精确跟随小米 HyperOS 当前主题商店设置的自定义系统字体。

请判断 A 和 B 各自是否能在不 Root、不读取受保护字体文件的情况下实现。

输出内容：

- 当前字体系统的调用链；
- 关键文件及作用；
- 关键函数、组件、状态字段和 CSS 变量；
- 推荐实现方案；
- 可能的兼容性问题；
- 需要新增或修改的文件清单；
- 建议的测试方案。

不要写代码，不要提交修改。所有判断必须基于当前仓库源码，不要凭经验猜测。