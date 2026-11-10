# Changelog

All notable changes to Clip-Vault will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
