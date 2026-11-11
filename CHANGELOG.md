# Changelog

All notable changes to Clip-Vault will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- 🪟 **Mini clipboard popup**: the global shortcut now opens a compact popup next to the cursor (like the Windows clipboard) instead of the full app. Search, pick with ↑ ↓ and press Enter or click to paste; Esc or clicking elsewhere closes it, ⤢ opens the full app

## [0.3.0] - 2026-11-10

### Added
- ⌨️ **Global shortcut** (default `Ctrl + Alt + V`) opens Clip-Vault from any app; pick a clip with ↑ ↓ and press Enter to paste it straight into the previous app. Can be turned off or changed in Settings
- 🔒 **Encrypted history**: clips, images and thumbnails are encrypted with AES-256-GCM; the key is kept in Windows Credential Manager / macOS Keychain. Existing history is encrypted automatically on first start
- 🌐 **Language setting**: English and Bahasa Indonesia (or follow the system language), including the tray menu

## [0.2.1] - 2026-11-10

### Added
- ℹ️ **About section** in Settings showing the app version and a link to the GitHub repository
- 🔄 **In-app updates**: "Check for updates" button with one-click download & install and a progress bar
- 🔔 **Automatic update check on startup** with an update banner (can be turned off in Settings)
- 🛠️ `scripts/build-release.ps1` for building signed releases locally, including `latest.json`

## [0.2.0] - 2026-11-10

### Added
- 📷 **Full image support** - Capture, preview, and copy images from clipboard
- 🖼️ **Image preview thumbnails** in clipboard history
- 🔄 **Copy images back to clipboard** with single click
- 🔔 **Toast notifications** for copy feedback
- 🎨 **Improved UI/UX** with modern minimalist design
- 🚫 **Duplicate detection** for images using hash-based comparison
- 📊 **PNG encoding/decoding** for efficient image storage
- ⚙️ **Settings menu**: launch at startup, start minimized, close to tray, auto-delete history (never/1/7/30/90 days), maximum clips limit, image capture toggle, and System/Light/Dark theme

### Changed
- ⚡ Enhanced clipboard monitoring to prioritize image capture
- 🎯 Better error handling with visual feedback
- 💾 Improved database storage for binary image data

### Fixed
- 🐛 Fixed Tauri v2 frontend API compatibility issues
- 🔧 Resolved clipboard monitoring race conditions

## [0.1.0] - 2026-11-09

### Added
- 📋 Basic clipboard history management
- 🔍 Search functionality for clipboard entries
- 📌 Pin favorite clipboard items
- 💾 SQLite persistent storage
- 🗑️ Delete and clear history features
- 🔄 Auto-refresh on clipboard changes
- 🎨 Modern UI with dark/light theme support
- 🖥️ System tray integration
- 📊 Statistics dashboard
- 🔐 Secure local storage in AppData

### Technical
- Built with Tauri v2.12.5
- Rust backend with arboard clipboard library
- SQLite database with rusqlite
- Cross-platform support (Windows, macOS, Linux)

[unreleased]: https://github.com/RobbiAbd/clip-vault/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/RobbiAbd/clip-vault/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/RobbiAbd/clip-vault/releases/tag/v0.1.0
