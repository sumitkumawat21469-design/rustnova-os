# RustNova OS - QEMU UEFI Launcher
$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BaseDir = Split-Path -Parent $ScriptDir
$QemuPath = "C:\Users\admin\.gemini\antigravity\scratch\tools\qemu\qemu-system-x86_64.exe"
$OvmfCode = "C:\Users\admin\.gemini\antigravity\scratch\tools\qemu\share\edk2-x86_64-code.fd"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "  Launching RustNova OS under QEMU (UEFI Firmware)" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

& $QemuPath `
    -drive "if=pflash,format=raw,readonly=on,file=$OvmfCode" `
    -drive "format=raw,file=fat:rw:$BaseDir\esp" `
    -serial stdio `
    -m 128M `
    -no-reboot
