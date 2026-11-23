# Rustnova-OS

**Rustnova-OS** is a custom operating system project developed using the Rust programming language. It includes a UEFI bootloader, kernel components, desktop UI, terminal, and system utilities.

## Project Overview

The goal of this project is to explore operating system development, boot processes, memory management, and system-level programming using Rust.

## Project Components

- **UEFI Bootloader:** Handles the initial boot process.
- **Kernel:** Core operating system components.
- **Desktop UI:** Graphical interface components.
- **Terminal:** Command-line interaction.
- **System Utilities:** Supporting system tools.

## Technologies Used

- Rust
- UEFI
- Python (build and utility scripts)
- Oracle VirtualBox
- QEMU

## Repository Structure

| Folder/File | Description |
|---|---|
| `src/` | Source code |
| `esp/` | EFI boot files |
| `scripts/` | Build and launch scripts |
| `docs/` | Project documentation |
| `results/` | Test results and screenshots |
| `traces/` | Boot logs and verification evidence |
| `rustnova-os.iso` | Bootable ISO image |
| `rustnova-os.vdi` | VirtualBox disk image |
| `rustnova-os.vmdk` | Virtual disk image |
| `rustnova-os.raw` | Raw disk image |

## Download

- [Download the ISO from Hugging Face](https://huggingface.co/datasets/sumitkumawat21469/rustnova-os-iso)
- [View the GitHub source repository](https://github.com/sumitkumawat21469-design/rustnova-os)

## VirtualBox Setup

1. Open Oracle VirtualBox.
2. Create a new virtual machine named `Rustnova-OS`.
3. Select the appropriate 64-bit operating system type.
4. Enable EFI in the virtual machine's System settings.
5. Attach the project ISO as a virtual optical disk.
6. Start the virtual machine and check the boot output.

*Note: This is an educational operating system project. Boot compatibility and functionality should be verified in the target virtual machine.*

## Project Status

This project contains the operating system source files, disk images, boot files, scripts, and verification logs. It is intended for learning and experimentation with Rust-based operating system development.

## Author

**Sumit Kumawat**

B.Tech – Computer Science and Engineering
