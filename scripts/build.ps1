# RustNova OS - Build Script
$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BaseDir = Split-Path -Parent $ScriptDir

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "  Building RustNova OS for x86_64 UEFI Target     " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

Set-Location $BaseDir
& "C:\Users\admin\.cargo\bin\cargo.exe" build --target x86_64-unknown-uefi

# Prepare ESP image directory
$EspBootDir = Join-Path $BaseDir "esp\EFI\BOOT"
New-Item -ItemType Directory -Path $EspBootDir -Force | Out-Null
Copy-Item (Join-Path $BaseDir "target\x86_64-unknown-uefi\debug\rustnova-os.efi") (Join-Path $EspBootDir "BOOTX64.EFI") -Force

Write-Host "[OK] UEFI Bootloader staged: esp\EFI\BOOT\BOOTX64.EFI" -ForegroundColor Green
