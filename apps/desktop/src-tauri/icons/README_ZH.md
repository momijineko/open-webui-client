# Tauri 应用图标

请在此目录下放置以下图标文件：

## Windows
- `icon.ico` - Windows 应用图标

## macOS
- `icon.icns` - macOS 应用图标

## Linux
- `32x32.png` - 32x32 像素 PNG 图标
- `128x128.png` - 128x128 像素 PNG 图标
- `128x128@2x.png` - 256x256 像素 PNG 图标（Retina）

## 生成图标

您可以使用以下工具生成图标：

1. **在线工具**: https://icon.kitchen/
2. **命令行工具**:
   ```bash
   # 安装 icon-tool
   cargo install icon-tool

   # 从 SVG 生成图标
   icon-tool icon.svg -o icons/
   ```

3. **ImageMagick**:
   ```bash
   convert icon.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
   ```

## 设计建议

- 使用简洁的设计
- 确保在小尺寸下也能清晰识别
- 使用品牌配色方案
- 避免过于复杂的细节
