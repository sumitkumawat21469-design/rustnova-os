# Rustnova OS - VirtualBox Automated VM Creator & Launcher
# Strictly manages ONLY the "Rustnova-OS" virtual machine.
# NEVER touches or modifies "Rust_Micro_OS" or any other existing VM.

$ErrorActionPreference = "Stop"

$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
if (!(Test-Path $vbox)) {
    Write-Error "VirtualBox not found at: $vbox"
    exit 1
}

$vmName = "Rustnova-OS"
$projectRoot = "C:\Users\admin\.gemini\antigravity\scratch\rustnova-os"
$vdiPath = "$projectRoot\rustnova-os.vdi"
$logPath = "$projectRoot\traces\virtualbox_serial.log"
$buildScript = "$projectRoot\scripts\build_bootable_disk.py"
$python = "C:\Program Files\Python313\python.exe"

Write-Host "========================================================="
Write-Host "  RUSTNOVA OS - VIRTUALBOX LAUNCHER                      "
Write-Host "========================================================="

# 1. Verify VDI disk existence; build immediately if missing
if (!(Test-Path $vdiPath) -or (Get-Item $vdiPath).Length -eq 0) {
    Write-Host "[DISK] rustnova-os.vdi not found or zero-length. Generating now..."
    & $python $buildScript
    if ($LASTEXITCODE -ne 0 -or !(Test-Path $vdiPath)) {
        Write-Error "FATAL: Failed to generate $vdiPath. Exiting immediately."
        exit 1
    }
}

$vdiItem = Get-Item $vdiPath
Write-Host "[DISK-OK] Verified VDI image: $vdiPath ($($vdiItem.Length) bytes)"

# 2. Check VM status - Strictly operate ONLY on "Rustnova-OS"
# SAFETY GUARANTEE: Never touch or alter "Rust_Micro_OS" or any other user VM
$allVms = & $vbox list vms
$targetExists = $allVms | Where-Object { $_ -match "^`"$vmName`"" }

if ($targetExists) {
    Write-Host "[VM] Existing `"$vmName`" found. Ensuring stopped state..."
    $runningVms = & $vbox list runningvms
    if ($runningVms | Where-Object { $_ -match "^`"$vmName`"" }) {
        Write-Host "[VM] Stopping running `"$vmName`"..."
        & $vbox controlvm $vmName poweroff 2>$null
        Start-Sleep -Seconds 2
    }
} else {
    Write-Host "[1/5] Creating 64-bit Virtual Machine: $vmName..."
    & $vbox createvm --name $vmName --ostype "Other_64" --register
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FATAL: Failed to create VirtualBox VM '$vmName'."
        exit 1
    }
}

# 3. Configure Motherboard, Memory, Graphics, and EFI Firmware
Write-Host "[2/5] Configuring EFI Firmware, 256 MB RAM, VBoxSVGA graphics (64MB VRAM)..."
& $vbox modifyvm $vmName --firmware efi
if ($LASTEXITCODE -ne 0) { Write-Error "FATAL: Failed to enable EFI firmware."; exit 1 }

& $vbox modifyvm $vmName --memory 256
& $vbox modifyvm $vmName --cpus 1
& $vbox modifyvm $vmName --vram 64
& $vbox modifyvm $vmName --graphicscontroller vboxsvga

# 4. Configure Serial Port COM1 (0x3F8, IRQ 4)
Write-Host "[3/5] Configuring COM1 serial capture to: $logPath..."
& $vbox modifyvm $vmName --uart1 0x3F8 4
& $vbox modifyvm $vmName --uartmode1 file $logPath

# 5. Configure SATA Controller and Bootable VDI Disk
Write-Host "[4/5] Verifying SATA storage controller and attaching bootable VDI..."
$vmInfo = & $vbox showvminfo $vmName --machinereadable
if ($vmInfo -notmatch 'storagecontrollername\d+="SATA"') {
    & $vbox storagectl $vmName --name "SATA" --add sata --controller IntelAhci --bootable on
    if ($LASTEXITCODE -ne 0) { Write-Error "FATAL: Failed to add SATA controller."; exit 1 }
}

& $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type hdd --medium $vdiPath
if ($LASTEXITCODE -ne 0) {
    & $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type hdd --medium none 2>$null
    & $vbox storageattach $vmName --storagectl "SATA" --port 0 --device 0 --type hdd --medium $vdiPath
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FATAL: Failed to attach VDI disk ($vdiPath) to SATA controller. Exiting."
        exit 1
    }
}

# 6. Start Virtual Machine
Write-Host "[5/5] Starting $vmName in VirtualBox..."
& $vbox startvm $vmName
if ($LASTEXITCODE -ne 0) {
    Write-Error "FATAL: Failed to start VM '$vmName'."
    exit 1
}

Write-Host "========================================================="
Write-Host "  SUCCESS: Rustnova OS is now booting in VirtualBox!     "
Write-Host "========================================================="
Write-Host "Target Disk: $vdiPath ($($vdiItem.Length) bytes)"
Write-Host "Serial Log:  $logPath"
