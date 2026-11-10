# 📋 Versioning & Release Strategy

## Semantic Versioning

Clip-Vault follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH (e.g., 1.2.3)
```

### Version Bump Rules

- **MAJOR** (X.0.0): Breaking changes, API incompatibility
  - Example: Remove features, change data format that breaks existing installs
  
- **MINOR** (0.X.0): New features, backward compatible
  - Example: Add image support, new UI features
  - Current: `v0.1.0` → `v0.2.0` (image support added)
  
- **PATCH** (0.0.X): Bug fixes, minor improvements
  - Example: Fix crashes, UI tweaks, performance improvements

### Pre-1.0.0 Note
- Version 0.x.x indicates the project is in initial development
- API may change without warning
- v1.0.0 will be released when the API is stable and production-ready

## Git Branching Strategy

```
main (production)          ─────v0.1.0──────────────v0.2.0─────→
                                  ↑                     ↑
                                  │ merge               │ merge
                                  │                     │
develop (integration)      ───────┴────[features]──────┴────────→
                                            ↑
                                            │ merge
                                            │
feature/image-support      ────────────────┘
```

### Branch Types

1. **`main`** - Production releases only
   - Only merge from `develop` or hotfix branches
   - Every commit should be tagged with version
   - Protected branch (no direct commits)

2. **`develop`** - Integration/beta branch
   - Default development branch
   - Merge feature branches here first
   - Test thoroughly before merging to `main`

3. **`feature/*`** - Individual features (optional)
   - Branch from `develop`
   - Naming: `feature/image-support`, `feature/keyboard-shortcuts`
   - Merge back to `develop` when complete

4. **`hotfix/*`** - Emergency fixes for production
   - Branch from `main`
   - Merge to both `main` AND `develop`
   - Bump PATCH version

## Release Process

### 1. Development Phase (on `develop` branch)
```bash
git checkout develop
# Make changes, commit regularly
git add .
git commit -m "feat: add image capture support"
git commit -m "feat: add image copy functionality"
git commit -m "feat: add toast notifications"
```

### 2. Prepare Release
```bash
# Update version in files
# - Cargo.toml: version = "0.2.0"
# - tauri.conf.json: "version": "0.2.0"
# - CHANGELOG.md: Add release notes

git add Cargo.toml tauri.conf.json CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin develop
```

### 3. Merge to Main
```bash
git checkout main
git pull origin main
git merge develop
git push origin main
```

### 4. Create Tag (Triggers CI/CD)
```bash
git tag -a v0.2.0 -m "Release v0.2.0: Image support and improvements"
git push origin v0.2.0
```

### 5. GitHub Actions Auto-Build
- Push tag `v0.2.0` triggers `.github/workflows/release.yml`
- Builds for Windows, macOS, Linux automatically
- Creates GitHub Release with all binaries
- Users can download without building

## Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
[optional footer]
```

### Types:
- `feat:` - New feature (MINOR bump)
- `fix:` - Bug fix (PATCH bump)
- `docs:` - Documentation only
- `style:` - Code style (formatting, no code change)
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `test:` - Adding tests
- `chore:` - Build process, dependencies

### Examples:
```bash
git commit -m "feat: add image capture from clipboard"
git commit -m "fix: resolve Tauri v2 API compatibility issue"
git commit -m "docs: update installation instructions"
git commit -m "chore: bump dependencies to latest versions"
```

## Checking Current Status

```bash
# Check current version
grep version Cargo.toml
grep version tauri.conf.json

# Check branch
git branch

# Check uncommitted changes
git status

# Check commit history
git log --oneline --graph --decorate --all
```

## Quick Reference

```bash
# Create feature branch
git checkout -b feature/my-feature develop

# Merge feature to develop
git checkout develop
git merge feature/my-feature

# Release to production
git checkout main
git merge develop
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main --tags

# Hotfix
git checkout -b hotfix/critical-bug main
# Fix bug
git checkout main
git merge hotfix/critical-bug
git checkout develop
git merge hotfix/critical-bug
git tag -a v0.2.1 -m "Hotfix v0.2.1"
git push origin main develop --tags
```

## Next Release Planning

### v0.3.0 (Future)
- Keyboard shortcuts
- Custom categories
- Cloud sync (optional)

### v0.4.0 (Future)
- Rich text formatting
- File attachments
- Advanced search filters

### v1.0.0 (Stable Release)
- API stable
- Full documentation
- Production-ready
