已完成只读分析。基线为：

- 本地仓库：`C:\Projects\readest`
- 分支：`main`
- 提交：`fcdc6567e0556c2ae2259c5d61737fbec66fa375`
- foliate-js 子模块：`90764e1509715e2ce026f0c142a628342fc7144b`
- 工作区无修改、无未提交差异

## 核心结论

Readest 的 EPUB 正文不是 Android `TextView` 或 Compose 渲染，而是：

```text
React 设置界面
  → ViewSettings / Zustand
  → settings.json 或单书配置
  → getStyles()
  → foliate-view
  → foliate-paginator
  → 每章节一个 iframe
  → 向 iframe <head> 注入 CSS
  → font-family / @font-face 应用到 EPUB HTML
```

Android 外层是 Tauri v2 创建的原生 Android `WebView`，正文内层又是 foliate-js 创建的 iframe。

“Android 默认系统字体”（需求 A）可以不使用原生桥、不读取字体文件实现，建议直接使用 CSS 通用字体族 `system-ui, sans-serif`。

“精确跟随 HyperOS 主题商店当前字体”（需求 B）在当前架构和现有桥接能力下不能可靠保证。`system-ui` 可能在某些 HyperOS 版本上间接跟随，但仓库没有检测“当前主题字体”的接口，也无法从现有字体枚举结果判断哪一个字体正在生效。

---

## 一、当前字体系统调用链

### 1. 设置页面

主入口：

- [SettingsDialog.tsx](C:/Projects/readest/apps/readest-app/src/components/settings/SettingsDialog.tsx:443)  
  将 `font` 设置页映射到 `FontPanel`。
- [FontPanel.tsx](C:/Projects/readest/apps/readest-app/src/components/settings/FontPanel.tsx:100)  
  阅读字体的主要设置页面。
- [FontDropDown.tsx](C:/Projects/readest/apps/readest-app/src/components/settings/FontDropDown.tsx:1)  
  字体下拉列表、字体预览、系统字体二级列表。
- [CustomFonts.tsx](C:/Projects/readest/apps/readest-app/src/components/settings/CustomFonts.tsx:26)  
  导入、显示、选择和删除自定义字体。
- [FontLayoutPanel.tsx](C:/Projects/readest/apps/readest-app/src/app/reader/components/footerbar/FontLayoutPanel.tsx:38)  
  手机阅读底部栏中的字体/布局快捷入口；不是最终字体渲染器。

`FontPanel` 当前提供：

- 默认字体类型：`Serif` / `Sans-serif`
- CJK 字体
- Serif 字体
- Sans-serif 字体
- Monospace 字体
- 自定义字体管理
- `Override Book Font`
- 字号、最小字号、字重

### 2. 设置保存

选择字体后，`FontPanel` 的各个 `useEffect` 调用：

```text
saveViewSettings(envConfig, bookKey, 字段名, 字体字符串)
```

主要字段：

```ts
interface BookFont {
  serifFont: string;
  sansSerifFont: string;
  monospaceFont: string;
  defaultFont: string;
  defaultCJKFont: string;
  defaultFontSize: number;
  minimumFontSize: number;
  fontWeight: number;
}
```

定义见 [book.ts](C:/Projects/readest/apps/readest-app/src/types/book.ts:249)。

字体相关状态不是枚举或字体对象，而是普通字符串：

- `defaultFont`: `"Serif"` 或 `"Sans-serif"`
- `serifFont`: 例如 `"Bitter"`
- `sansSerifFont`: 例如 `"Roboto"`
- `monospaceFont`: 例如 `"Consolas"`
- `defaultCJKFont`: 例如 `"LXGW WenKai GB Screen"`
- `overrideFont`: `boolean`

默认值在 [constants.ts](C:/Projects/readest/apps/readest-app/src/services/constants.ts:275)：

```text
serifFont       = Bitter
sansSerifFont   = Roboto
monospaceFont   = Consolas
defaultFont     = Serif
defaultCJKFont  = LXGW WenKai GB Screen
overrideFont    = false
```

移动端会把 `defaultFont` 覆盖为 `Sans-serif`。

### 3. 全局和单书持久化

全局设置：

```text
settings.json
└─ globalViewSettings
   ├─ defaultFont
   ├─ serifFont
   ├─ sansSerifFont
   ├─ monospaceFont
   ├─ defaultCJKFont
   └─ overrideFont
```

加载和保存位置：

- [settingsService.ts](C:/Projects/readest/apps/readest-app/src/services/settingsService.ts:106)
- [settingsService.ts](C:/Projects/readest/apps/readest-app/src/services/settingsService.ts:174)
- [helpers/settings.ts](C:/Projects/readest/apps/readest-app/src/helpers/settings.ts:32)

单书设置：

- 存放在该书的配置 JSON 中；
- 只保存与 `globalViewSettings` 不同的字段；
- 打开书籍时使用 `{...globalViewSettings, ...bookViewSettings}` 合并。

相关代码：

- [bookService.ts](C:/Projects/readest/apps/readest-app/src/services/bookService.ts:739)
- [serializer.ts](C:/Projects/readest/apps/readest-app/src/utils/serializer.ts:27)
- [serializer.ts](C:/Projects/readest/apps/readest-app/src/utils/serializer.ts:69)

需要注意：普通 settings replica 白名单目前不包含这些字体字段。自定义字体文件则有自己独立的 `font` replica 同步机制。

### 4. 生成阅读 CSS

`saveViewSettings()` 在设置变化后调用：

```ts
view.renderer.setStyles(getStyles(viewSettings))
```

见 [helpers/settings.ts](C:/Projects/readest/apps/readest-app/src/helpers/settings.ts:45)。

`getStyles()` 位于 [style.ts](C:/Projects/readest/apps/readest-app/src/utils/style.ts:842)，内部调用 `getFontStyles()`，生成：

```css
html {
  --serif: ... , serif;
  --sans-serif: ... , sans-serif;
  --monospace: ... , monospace;
  --font-size: ...;
  --min-font-size: ...;
  --font-weight: ...;
}

html {
  font-family: var(--serif); /* 或 --sans-serif */
}
```

关键 CSS 变量：

- `--serif`
- `--sans-serif`
- `--monospace`
- `--font-size`
- `--min-font-size`
- `--font-weight`

最终默认正文族由 `defaultFont` 决定：

```text
Serif      → var(--serif)
Sans-serif → var(--sans-serif)
```

### 5. foliate-js 和 iframe 注入

[FoliateViewer.tsx](C:/Projects/readest/apps/readest-app/src/app/reader/components/FoliateViewer.tsx:661)：

1. 动态导入 `foliate-js/view.js`
2. 创建 `<foliate-view>`
3. `view.open(bookDoc)`
4. 调用 `view.renderer.setStyles(getStyles(...))`

[view.js](C:/Projects/readest/packages/foliate-js/view.js:215) 根据书籍布局创建：

- 流式 EPUB：`<foliate-paginator>`
- 固定布局：`<foliate-fxl>`

[paginator.js](C:/Projects/readest/packages/foliate-js/paginator.js:560) 为章节创建 iframe：

```js
#iframe = document.createElement('iframe')
```

每个章节载入后，在 iframe 的 `<head>` 中创建用户样式节点；`setStyles()` 将 `getStyles()` 的结果写入该 `<style>`。见：

- [paginator.js](C:/Projects/readest/packages/foliate-js/paginator.js:2882)
- [paginator.js](C:/Projects/readest/packages/foliate-js/paginator.js:3309)

因此 `font-family` 最终生效位置是“每个 EPUB 章节 iframe 内的注入 `<style>`”。

---

## 二、三类字体如何管理

### 内置字体

内置字体并不等于 APK 内置字体文件。主体是：

1. 常量中的候选字体名称；
2. Google Fonts / CDN 在线样式表；
3. 一组系统本地字体回退名称。

主要文件：

- [constants.ts](C:/Projects/readest/apps/readest-app/src/services/constants.ts:496)
- [fonts.ts](C:/Projects/readest/apps/readest-app/src/styles/fonts.ts:7)

`mountAdditionalFonts()` 会往主文档和章节 iframe 注入在线字体 `<link>` 以及部分 CJK `@font-face`。Android 离线或网络/CSP失败时会继续沿 CSS 字体链回退。

仓库虽然定义了 `ANDROID_FONTS`，但它当前只被测试引用，没有被 `FontPanel` 使用。

### 自定义字体

流程：

```text
用户选择 TTF/OTF/WOFF/WOFF2
 → importFont()
 → 解析字体 name/family/style/weight
 → 写入 Fonts/<bundleDir>/<filename>
 → CustomFont[] 保存到 settings.customFonts
 → 读取文件为 Blob
 → URL.createObjectURL()
 → 生成 @font-face
 → 注入主文档和章节 iframe
```

关键数据结构 `CustomFont` 在 [fonts.ts](C:/Projects/readest/apps/readest-app/src/styles/fonts.ts:131)，包含：

- `id`
- `name`
- `path`
- `family`
- `style`
- `weight`
- `variable`
- `contentId`
- `bundleDir`
- `byteSize`
- 运行时的 `blobUrl`、`loaded`、`error`

关键实现：

- [fontService.ts](C:/Projects/readest/apps/readest-app/src/services/fontService.ts:26)：导入和落盘
- [customFontStore.ts](C:/Projects/readest/apps/readest-app/src/store/customFontStore.ts:81)：Zustand 字体状态
- [fonts.ts](C:/Projects/readest/apps/readest-app/src/styles/fonts.ts:217)：生成 `@font-face`
- [FoliateViewer.tsx](C:/Projects/readest/apps/readest-app/src/app/reader/components/FoliateViewer.tsx:918)：装载到主文档和现有 iframe
- [font.ts](C:/Projects/readest/apps/readest-app/src/services/sync/adapters/font.ts:46)：独立字体同步

`blobUrl`、加载状态和错误不会持久化。

### EPUB 自带字体

foliate-js 会识别 EPUB 中的：

- `.woff`
- `.woff2`
- `.ttf`
- `.otf`

见 [epub.js](C:/Projects/readest/packages/foliate-js/epub.js:469)。

EPUB CSS 中的字体 URL 会由 `loadHref()`/`loadItem()` 解析，字体文件转成可访问的 Blob URL，原书 `@font-face` 保留并参与 CSS 层叠。

另外，`FoliateViewer` 有一条特殊兼容逻辑：如果 EPUB 引用 `fonts/文件名`，而用户自定义字体中存在同名文件，可以用该自定义字体的 Blob URL替代缺失资源。见 [FoliateViewer.tsx](C:/Projects/readest/apps/readest-app/src/app/reader/components/FoliateViewer.tsx:693)。

---

## 三、EPUB CSS 会不会覆盖用户选择

会，取决于 `overrideFont`。

### `overrideFont = false`

Readest 给 `html` 设置默认字体，但不加 `!important`。源码明确标注这是“低于电子书内置字体样式”的策略。

因此：

- EPUB 对 `body`、段落或具体元素指定的字体可能覆盖用户默认字体；
- EPUB 的内嵌 `@font-face` 会正常生效；
- 用户字体主要是缺省/继承字体。

不过 `transformStylesheet()` 会把 EPUB CSS 中的通用族重写：

```text
serif      → var(--serif, serif)
sans-serif → var(--sans-serif, sans-serif)
monospace  → var(--monospace, monospace)
```

还会将 `body { font-family: serif }` 或 `sans-serif` 改为 `unset`，使 Readest 默认字体更容易接管。见 [style.ts](C:/Projects/readest/apps/readest-app/src/utils/style.ts:1043)。

### `overrideFont = true`

注入：

```css
html,
html body {
  font-family: var(...) !important;
}
```

并对多数正文子元素执行现有的强制重置规则。这一模式用于压过 EPUB 的字体声明。

兼容风险包括：

- 图标字体、数学字体、特殊符号字体可能被错误覆盖；
- EPUB 在具体元素上使用 `!important` 时仍需比较层叠来源和选择器；
- 代码块被单独保留为 `--monospace`；
- 固定布局 EPUB/PDF 的处理不同，不应把正文系统字体功能扩大到固定布局。

---

## 四、各平台是否共用逻辑

核心字体逻辑共用：

- React 设置 UI
- `ViewSettings`
- `getStyles()`
- foliate-js
- iframe CSS 注入
- 自定义字体 Blob 和 `@font-face`

平台差异主要在系统字体候选来源：

| 平台 | 当前行为 |
|---|---|
| Windows/macOS/Linux | 常量默认列表，并通过 Tauri 桌面桥枚举已安装字体 |
| iOS | `IOS_FONTS`，且非 Android 的 Tauri 分支会调用字体桥 |
| Android | `defaultSysFonts = []`，UI 明确不调用系统字体枚举 |
| Web | 无 Tauri 系统字体枚举，只能依赖 CSS 可解析字体和在线字体 |

因此不是五个平台完全独立，也不是百分之百相同：渲染和状态共用，系统字体发现逻辑分平台。

---

## 五、Android 原生桥现状

当前项目明确使用 Tauri v2，不是 Capacitor。

Android 代码包括：

- Kotlin `MainActivity`
- Kotlin 原生桥插件
- Rust Tauri mobile wrapper
- Android WebView

关键文件：

- [MainActivity.kt](C:/Projects/readest/apps/readest-app/src-tauri/gen/android/app/src/main/java/com/bilingify/readest/MainActivity.kt:26)
- [NativeBridgePlugin.kt](C:/Projects/readest/apps/readest-app/src-tauri/plugins/tauri-plugin-native-bridge/android/src/main/java/NativeBridgePlugin.kt:653)
- [mobile.rs](C:/Projects/readest/apps/readest-app/src-tauri/plugins/tauri-plugin-native-bridge/src/mobile.rs:101)
- [bridge.ts](C:/Projects/readest/apps/readest-app/src/utils/bridge.ts:173)

Android 桥已经有 `get_sys_fonts_list`：

- Android 10+ 使用 `SystemFonts.getAvailableFonts()`
- 旧版本尝试扫描 `/system/fonts`、`/system/font`、`/data/fonts`
- 返回的是字体文件基本名
- 不返回“当前默认字体”
- 不返回“当前主题字体”
- 不建立真实 family/style 映射

而 `FontPanel` 有意使用：

```ts
isTauriAppPlatform() && appService && !appService.isAndroidApp
```

所以该 Android 命令虽然存在，目前不会被字体设置页调用。

---

## 六、system-ui 等现状

阅读正文主链当前没有使用 `system-ui` 或 `ui-sans-serif`。

正文相关回退主要是：

- `serif`
- `sans-serif`
- `monospace`

`system-ui` 仅出现在 foliate-js 自带的独立 `reader.html` UI 中，不属于 Readest 当前正文注入链。

其他出现的 `-apple-system`、`BlinkMacSystemFont` 等主要用于：

- 网页剪藏提示 UI
- EPUB 转换生成模板
- 封面生成
- 非正文工具页面

---

## 七、A 与 B 的可实现性

### A. Android 默认 UI 字体

结论：可以，不需要 Root，不需要读取任何字体文件，也不需要原生桥。

最直接的正文 CSS 是：

```css
font-family: system-ui, sans-serif;
```

它让 Android WebView/Chromium 自己选择平台 UI 通用字体。这里不能把 `system-ui` 放进现有普通字体字符串后直接处理，因为 `buildFontFamilyLists()` 会给每个字体名称加引号：

```css
"system-ui"
```

带引号后它会被当作名为 `system-ui` 的普通字体，而不是 CSS 通用字体族。

因此实现时必须把 `system-ui` 作为特殊通用族输出，保持不加引号。

### B. 精确跟随 HyperOS 主题字体

结论：在当前仓库能力下不能可靠实现或保证；无 Root、无受保护字体读取时尤其无法把它当作一个“精确功能”承诺。

源码依据：

- 没有查询当前 Android 默认 `Typeface` 的桥；
- 没有 HyperOS/MIUI 主题服务接口；
- `SystemFonts.getAvailableFonts()` 只枚举可用字体，无法指出当前使用哪个；
- 旧版 `/data/fonts` 扫描可能因权限不可读；
- 文件名也不等于 WebView 可用的 CSS family 名；
- 没有调用 `WebSettings.setStandardFontFamily()`；
- 没有将原生 `Typeface` 注入 WebView/iframe 的机制。

如果 HyperOS 将主题字体映射到了 Android/Chromium 的默认通用字体，`system-ui` 或 `sans-serif` 可能自然跟随。这属于 OEM/WebView 行为，不是当前代码可以验证或保证的“精确跟随”。

要严格实现 B，至少需要以下之一：

- HyperOS 提供稳定、公开、无需特殊权限的“当前主题字体”API；
- 用户主动导入可访问的字体文件；
- 读取或复制主题字体文件；
- OEM 专用且版本相关的实现。

当前仓库没有前两者之外的可靠基础。

---

## 八、推荐的最小安全方案

建议把“系统字体”设计成 `defaultFont` 的第三种模式，而不是伪装成某个具体字体文件：

```text
defaultFont:
  Serif
  Sans-serif
  System
```

当选择 `System` 时：

```text
正文基础字体 → system-ui, sans-serif
CJK 缺字      → 由 Android/WebView 系统回退处理
不加载文件    → 不生成 @font-face
不枚举字体    → 不需要 get_sys_fonts_list
```

原因：

- 不触碰 Kotlin、Rust、Tauri 权限或字体目录；
- 不依赖不可靠的 Android 字体文件名枚举；
- 保持现有全局/单书保存结构；
- `defaultFont` 已经是普通 `string`，无需数据迁移；
- Windows/macOS/Linux/Web 即使收到该值也能安全退化为各自的 `system-ui`；
- 自定义字体和 EPUB 字体加载链不受影响；
- 可继续使用现有 `overrideFont` 控制是否覆盖原书字体。

不建议把 `system-ui` 直接加入 `sansSerifFont` 普通字体数组，因为现有代码会自动加引号。

---

## 九、预计需要修改/新增的文件

仅列后续实施范围；本次未修改。

必需：

1. `src/components/settings/FontPanel.tsx`  
   增加“系统字体”选项及预览逻辑。

2. `src/utils/style.ts`  
   将 `defaultFont === System` 映射为未加引号的 `system-ui, sans-serif`；同步更新 `getBaseFontFamily()`。

3. `src/app/reader/components/paragraph/ParagraphOverlay.tsx`  
   当前自行判断 Serif/Sans-serif，需要识别 System，或改为复用统一解析函数。

4. `public/locales/*/translation.json` 或 i18n 源流程  
   新增“System Font”等文本。至少应覆盖英文和中文。

测试：

5. `src/__tests__/utils/style-get-styles.test.ts`
6. `src/__tests__/utils/style-base-font-family.test.ts`
7. `src/__tests__/components/settings/...`  
   增加设置选择和保存测试。
8. `src/__tests__/components/rsvp-overlay-context.test.tsx`  
   验证 RSVP 顶层覆盖层使用系统字体链。
9. `src/__tests__/utils/style.test.ts`  
   验证 EPUB CSS 重写和 `overrideFont` 下的 system 模式。

正常情况下不需要修改：

- Kotlin `NativeBridgePlugin.kt`
- `MainActivity.kt`
- Rust mobile bridge
- foliate-js
- 自定义字体存储和同步代码
- Tauri 权限配置

---

## 十、建议测试方案

### 单元测试

- `defaultFont = System` 时 CSS 必须包含 `system-ui, sans-serif`。
- 不能输出 `"system-ui"`。
- `overrideFont=false` 时不应意外增加 `!important`。
- `overrideFont=true` 时应对 `html/body` 使用系统字体链和既有强制覆盖逻辑。
- Serif/Sans-serif 原行为保持不变。
- RSVP、段落模式等 iframe 外正文辅助 UI 使用相同系统字体链。
- 全局设置、单书设置序列化/反序列化能保留 `"System"`。

### EPUB 样本矩阵

至少准备：

1. 无字体 CSS 的普通 EPUB。
2. `body { font-family: serif; }`。
3. `body { font-family: sans-serif; }`。
4. 指定具体字体名的 EPUB。
5. 内含 `@font-face` 的 EPUB。
6. 内联 `style="font-family:..."` 的 EPUB。
7. EPUB 字体声明带 `!important`。
8. 中文、英文、中英混排、Emoji、特殊符号和代码块。
9. 横排和竖排。
10. 流式 EPUB与固定布局 EPUB分别测试。

### Android 设备矩阵

- AOSP/Pixel 系统字体。
- 小米 HyperOS 默认字体。
- HyperOS 主题商店字体启用前后。
- 不同 Android System WebView 版本。
- Android 9 及 Android 10+。
- 离线状态，确认系统字体模式不依赖 Google Fonts/CDN。

HyperOS 测试结果应区分：

- `system-ui` 是否使用默认 Roboto/MiSans；
- 更换主题字体后是否自动变化；
- 应用重启后是否变化；
- WebView 更新前后是否一致。

产品文案宜写“系统字体”或“使用设备默认字体”，不应承诺“精确跟随主题商店字体”。