# Clip-Vault

A cross-platform clipboard manager built with Rust and Tauri, targeting Windows first with support for Linux and macOS.

## Features

- 📋 **Automatic Clipboard Monitoring** - Captures everything you copy
- 🔍 **Full-Text Search** - Quickly find clips in your history
- 📌 **Pin Important Clips** - Keep frequently used items at the top
- 🖼️ **Multi-Format Support** - Text, rich text, and images
- 💾 **Persistent Storage** - SQLite database for reliable storage
- 🎨 **Modern UI** - Clean and intuitive interface
- 🚀 **System Tray Integration** - Quick access from your system tray
- 🔒 **Local-Only** - All data stays on your machine

## Technology Stack

- **Backend**: Rust
- **Frontend**: HTML/CSS/JavaScript with Tauri
- **Database**: SQLite (rusqlite)
- **Clipboard Access**: clipboard-win (Windows), arboard (Linux/macOS)
- **GUI Framework**: Tauri

## Prerequisites

- Rust 1.70 or later
- Node.js (optional, for frontend tooling)
- Windows: Windows 10/11 with Visual Studio Build Tools
- Linux: WebKit2GTK development libraries
- macOS: Xcode Command Line Tools

### Windows Setup

Install Visual Studio Build Tools with C++ development tools:
```bash
# Download from: https://visualstudio.microsoft.com/downloads/
# Or use winget:
winget install Microsoft.VisualStudio.2022.BuildTools
```

### Linux Setup

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    libappindicator-gtk3-devel \
    librsvg2-devel

# Arch
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    libappindicator-gtk3 \
    librsvg
```

### macOS Setup

```bash
xcode-select --install
```

## Building

### Development Build

```bash
# Clone the repository
git clone <repository-url>
cd clip-vault

# Build and run in development mode
cargo tauri dev
```

### Production Build

```bash
# Build the application
cargo tauri build

# The installer will be in:
# Windows: target/release/bundle/msi/clip-vault_0.1.0_x64_en-US.msi
# Linux: target/release/bundle/deb/clip-vault_0.1.0_amd64.deb (or .AppImage)
# macOS: target/release/bundle/dmg/clip-vault_0.1.0_x64.dmg
```

## Installation

### Windows

1. Download the MSI installer from releases
2. Run the installer
3. Clip-Vault will start automatically and appear in the system tray

### Linux

```bash
# Debian/Ubuntu
sudo dpkg -i clip-vault_0.1.0_amd64.deb

# Or use AppImage
chmod +x clip-vault_0.1.0_x86_64.AppImage
./clip-vault_0.1.0_x86_64.AppImage
```

### macOS

1. Download the DMG from releases
2. Open the DMG and drag Clip-Vault to Applications
3. Launch from Applications or Spotlight

## Usage

### Basic Usage

1. **Automatic Capture**: Simply copy text, images, or files as usual. Clip-Vault will automatically save them.

2. **View History**: Click the system tray icon to open the main window and browse your clipboard history.

3. **Search**: Use the search bar to find specific clips.

4. **Copy Again**: Click on any clip to view it in detail, then click "Copy" to copy it to your clipboard.

5. **Pin Important Clips**: Click the pin icon to keep frequently used clips at the top.

6. **Delete Clips**: Click the delete button to remove unwanted clips from history.

### Keyboard Shortcuts (Future Feature)

- `Ctrl + Shift + V` - Open Clip-Vault
- `Ctrl + F` - Focus search
- `Escape` - Close window

### System Tray

- **Left Click**: Show/hide main window
- **Right Click**: Access menu
  - Show Clip-Vault
  - Quit

## Configuration

The application stores data in:
- Windows: `%APPDATA%\com.clipvault.app\clipboard.db`
- Linux: `~/.config/com.clipvault.app/clipboard.db`
- macOS: `~/Library/Application Support/com.clipvault.app/clipboard.db`

## Architecture

See [docs/architecture.md](docs/architecture.md) for detailed architecture documentation.

## Project Structure

```
clip-vault/
├── src/
│   ├── main.rs              # Application entry point
│   ├── models.rs            # Data models
│   ├── commands.rs          # Tauri IPC commands
│   ├── clipboard/           # Clipboard monitoring
│   │   ├── mod.rs
│   │   ├── windows_monitor.rs
│   │   └── generic_monitor.rs
│   └── storage/             # Database layer
│       └── mod.rs
├── ui/                      # Frontend
│   ├── index.html
│   ├── styles.css
│   ├── app.js
│   └── dist/                # Built UI files
├── icons/                   # Application icons
├── tauri.conf.json          # Tauri configuration
├── Cargo.toml               # Rust dependencies
└── build.rs                 # Build script
```

## Development

### Running Tests

```bash
cargo test
```

### Logging

Set the `RUST_LOG` environment variable for detailed logs:

```bash
# Windows
set RUST_LOG=debug
cargo tauri dev

# Linux/macOS
RUST_LOG=debug cargo tauri dev
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Check
cargo check
```

## Troubleshooting

### Windows: "VCRUNTIME140.dll not found"
Install the Visual C++ Redistributable:
https://support.microsoft.com/en-us/help/2977003/the-latest-supported-visual-c-downloads

### Linux: "Failed to load shared library libwebkit2gtk"
Install WebKit2GTK development libraries (see Linux Setup above)

### macOS: "Clip-Vault is damaged and can't be opened"
Run: `xattr -cr /Applications/Clip-Vault.app`

### Database Locked
Close other instances of Clip-Vault. Only one instance can run at a time.

## Future Enhancements

- [ ] Cloud synchronization
- [ ] Encrypted storage
- [ ] Advanced formatting preservation (HTML, RTF)
- [ ] Clipboard templates
- [ ] Customizable keyboard shortcuts
- [ ] Multiple clipboard slots (Ctrl+Shift+1-9)
- [ ] Auto-start on system boot
- [ ] Dark mode
- [ ] Clipboard statistics
- [ ] Import/Export functionality

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details

## Credits

Built with:
- [Tauri](https://tauri.app/) - Desktop application framework
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [clipboard-win](https://github.com/DoumanAsh/clipboard-win) - Windows clipboard API
- [arboard](https://github.com/1Password/arboard) - Cross-platform clipboard
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings

## Support

For issues, questions, or suggestions, please open an issue on GitHub.
