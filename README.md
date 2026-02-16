# GLB to FBX Converter 🎨

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A fast, efficient, and feature-rich command-line tool for batch converting GLB (glTF Binary) files to FBX format, written entirely in Rust.

**Created by: Trident_For_U**

## ✨ Features

- 🚀 **High Performance** - Pure Rust implementation with zero external dependencies on system libraries
- 📦 **Batch Processing** - Convert entire folders of GLB files in one go
- 🎯 **Progress Tracking** - Beautiful progress bars and colored output
- 🔄 **Smart Overwriting** - Automatically replaces existing FBX files
- 🌲 **Recursive Scanning** - Finds GLB files in all subdirectories
- 💾 **Preserve Structure** - Maintains mesh geometry, vertices, indices, and scene hierarchy
- ⚡ **Binary FBX Output** - Generates industry-standard FBX 7.4 binary files
- 🎨 **Colorful CLI** - Easy-to-read colored output with clear status indicators

## 📋 Requirements

- Rust 1.70 or higher
- Windows, macOS, or Linux

## 🔧 Installation

### Build from Source

```bash
git clone <repository-url>
cd GLB2FBX_RS
cargo build --release
```

The compiled executable will be available at:
- **Windows**: `target\release\glb2fbx.exe`
- **Linux/macOS**: `target/release/glb2fbx`

### Add to PATH (Optional)

**Windows:**
```powershell
# Copy to a directory in your PATH
copy target\release\glb2fbx.exe C:\Windows\System32\
```

**Linux/macOS:**
```bash
# Copy to a directory in your PATH
sudo cp target/release/glb2fbx /usr/local/bin/
```

## 🚀 Usage

### Basic Syntax

```bash
glb2fbx --input <INPUT_FOLDER> --output <OUTPUT_FOLDER>
```

### Examples

**Convert all GLB files from one folder to another:**
```bash
glb2fbx -i ./input_models -o ./output_models
```

**With full paths:**
```bash
glb2fbx --input "C:\Models\GLB" --output "C:\Models\FBX"
```

**Unix-style paths:**
```bash
glb2fbx -i ~/Downloads/models -o ~/Documents/converted
```

### Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--input <PATH>` | `-i` | Input folder containing GLB files (required) |
| `--output <PATH>` | `-o` | Output folder for converted FBX files (required) |
| `--help` | `-h` | Display help information |
| `--version` | `-V` | Display version information |

## 📊 Output Format

The tool generates **FBX 7.4 Binary** files compatible with:
- Autodesk Maya
- Autodesk 3ds Max
- Blender
- Unity
- Unreal Engine
- Cinema 4D
- And most other 3D software packages

### Preserved Data

- ✅ Vertex positions
- ✅ Polygon indices (triangulated)
- ✅ Scene graph hierarchy
- ✅ Mesh names
- ✅ Node transforms

### Current Limitations

- ⚠️ Materials and textures are not converted (geometry only)
- ⚠️ Animations are not supported
- ⚠️ Skeletal rigs are not supported

## 🎨 Example Output

```
╔════════════════════════════════════════╗
║        GLB to FBX Converter v1.0       ║
║         by Trident_For_U              ║
╚════════════════════════════════════════╝

→ Scanning for GLB files in: D:\Models\GLB
✓ Found 5 GLB file(s)

  █▓▒░ [████████████████████████████████████] 5/5 Converting model5.glb
  ✓ model1.glb → model1.fbx
  ✓ model2.glb → model2.fbx
  ✓ model3.glb → model3.fbx
  ✓ model4.glb → model4.fbx
  ✓ model5.glb → model5.fbx

══════════════════════════════════════════════
  Conversion Summary
══════════════════════════════════════════════
  ✓ 5 successfully converted
  → Total processed: 5
══════════════════════════════════════════════
```

## 🏗️ Technical Details

### Dependencies

- **gltf** - GLB file parsing and validation
- **fbxcel** - FBX binary format writer
- **walkdir** - Recursive directory traversal
- **clap** - Command-line argument parsing
- **colored** - Terminal color support
- **indicatif** - Progress bar rendering
- **anyhow** - Error handling and propagation

### Architecture

The converter uses a streaming approach:
1. Parse GLB binary files using the `gltf` crate
2. Extract mesh geometry and scene graph data
3. Write data directly to FBX binary format using `fbxcel`
4. No intermediate file formats or temporary files

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Improve documentation

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 👤 Author

**Trident_For_U**

## 🙏 Acknowledgments

- **fbxcel** - Excellent FBX library for Rust
- **gltf-rs** - Robust glTF parsing library
- Rust community for amazing tooling and support

## 📚 References

- [FBX File Format Specification](https://code.blender.org/2013/08/fbx-binary-file-format-specification/)
- [glTF 2.0 Specification](https://www.khronos.org/gltf/)
- [Autodesk FBX Documentation](https://help.autodesk.com/view/FBX/2020/ENU/)

---

**Made with ❤️ by Trident_For_U**
