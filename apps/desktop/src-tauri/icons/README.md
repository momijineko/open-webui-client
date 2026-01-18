# Tauri Application Icons

Please place the following icon files in this directory:

## Windows
- `icon.ico` - Windows application icon

## macOS
- `icon.icns` - macOS application icon

## Linux
- `32x32.png` - 32x32 pixel PNG icon
- `128x128.png` - 128x128 pixel PNG icon
- `128x128@2x.png` - 256x256 pixel PNG icon (Retina)

## Generating Icons

You can use the following tools to generate icons:

1. **Online Tool**: https://icon.kitchen/
2. **Command-line Tool**:
   ```bash
   # Install icon-tool
   cargo install icon-tool

   # Generate icons from SVG
   icon-tool icon.svg -o icons/
   ```

3. **ImageMagick**:
   ```bash
   convert icon.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
   ```

## Design Recommendations

- Use a simple, clean design
- Ensure clarity at small sizes
- Use brand color scheme
- Avoid overly complex details
