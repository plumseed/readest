请按照分析报告实现“系统字体”模式。

核心要求：

1. 新增 defaultFont = "System" 语义值。
2. 生成正文 CSS 时输出：
   font-family: system-ui, sans-serif;
3. system-ui 必须作为 CSS generic family 原样输出，不能加引号。
4. 小米 HyperOS、荣耀 MagicOS、Pixel、三星等 Android 设备共用同一实现，不硬编码任何厂商字体名称。
5. 不读取 /system/fonts、/data/fonts 或主题字体文件。
6. 不增加文件权限，不要求 Root，不使用未公开 OEM API。
7. 产品文案写“系统字体”或“设备默认字体”，不要承诺一定跟随主题商店字体。
8. 保留现有自定义字体导入功能，作为主题字体未传递给 WebView 时的备用方案。
9. 确保设置页面、EPUB iframe、ParagraphOverlay 和其他正文辅助视图都使用同一字体解析函数。
10. 增加测试，明确验证输出中是 system-ui 而不是 "system-ui"。

另外增加一个调试页面或开发日志，在 Android 真机上执行：
getComputedStyle(document.documentElement).fontFamily
并记录结果，帮助判断小米和荣耀设备上 WebView 最终解析出的字体族。

不要添加小米或荣耀专用代码，除非找到有官方文档、公开稳定且无需特殊权限的 API；如找到，先报告，不要直接集成。