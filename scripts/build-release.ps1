# Builds signed Windows installers plus the latest.json the in-app updater reads,
# and gathers everything to upload to a GitHub release in one folder.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 [-Notes "What changed"]

param(
    [string]$Notes = "See CHANGELOG.md for details."
)

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)

$version = (Get-Content tauri.conf.json -Raw | ConvertFrom-Json).version
$cargoVersion = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
if ($version -ne $cargoVersion) {
    throw "Version mismatch: tauri.conf.json has $version but Cargo.toml has $cargoVersion. Make them equal first."
}

$keyPath = Join-Path $env:USERPROFILE '.tauri\clip-vault.key'
if (-not (Test-Path $keyPath)) {
    throw "Signing key not found at $keyPath"
}
$env:TAURI_SIGNING_PRIVATE_KEY = $keyPath

# A running instance locks target\release\clip-vault.exe.
if (Get-Process clip-vault -ErrorAction SilentlyContinue) {
    Write-Host "Closing the running Clip-Vault so the build can replace its executable..."
    Get-Process clip-vault | Stop-Process -Force
}

Write-Host "Building Clip-Vault $version..."
cargo tauri build
if ($LASTEXITCODE -ne 0) {
    throw "cargo tauri build failed"
}

$bundle = 'target\release\bundle'
$setup = "Clip-Vault_${version}_x64-setup.exe"
$msi = "Clip-Vault_${version}_x64_en-US.msi"
$out = "target\release\upload-v$version"

Remove-Item $out -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory $out | Out-Null
Copy-Item "$bundle\nsis\$setup", "$bundle\nsis\$setup.sig", "$bundle\msi\$msi", "$bundle\msi\$msi.sig" $out

$manifest = [ordered]@{
    version   = $version
    notes     = $Notes
    pub_date  = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    platforms = [ordered]@{
        'windows-x86_64' = [ordered]@{
            signature = (Get-Content "$out\$setup.sig" -Raw).Trim()
            url       = "https://github.com/RobbiAbd/clip-vault/releases/download/v$version/$setup"
        }
    }
}
$outPath = (Resolve-Path $out).Path
[IO.File]::WriteAllText((Join-Path $outPath 'latest.json'), ($manifest | ConvertTo-Json -Depth 5))

Write-Host ""
Write-Host "Done. Upload every file in this folder to a GitHub release tagged v${version}:"
Write-Host "  $outPath"
Write-Host "Create it at: https://github.com/RobbiAbd/clip-vault/releases/new?tag=v$version"
Write-Host "Keep 'Set as the latest release' checked, or the updater won't find it."
