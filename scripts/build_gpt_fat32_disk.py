#!/usr/bin/env python3
"""
RustNova OS - Standard GPT + FAT32 UEFI Bootable Disk Image Generator.

Generates a fully compliant 64 MB GPT disk image with an EFI System Partition
(GUID C12A7328-F81F-11D2-BA4B-00A0C93EC93B) formatted as true FAT32, containing:
  - \\startup.nsh
  - \\EFI\\BOOT\\BOOTX64.EFI

Converts the raw image to VirtualBox VDI (.vdi) and VMware/VirtualBox VMDK (.vmdk).
"""

import os
import struct
import subprocess
import shutil
import zlib
import uuid

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
EFI_SOURCE = os.path.join(BASE_DIR, "target", "x86_64-unknown-uefi", "debug", "rustnova-os.efi")
ESP_DIR = os.path.join(BASE_DIR, "esp")
RAW_OUTPUT = os.path.join(BASE_DIR, "rustnova-os.raw")
VDI_OUTPUT = os.path.join(BASE_DIR, "rustnova-os.vdi")
VMDK_OUTPUT = os.path.join(BASE_DIR, "rustnova-os.vmdk")

QEMU_IMG = r"C:\Users\admin\.gemini\antigravity\scratch\tools\qemu\qemu-img.exe"

def build_gpt_fat32_disk():
    if not os.path.exists(EFI_SOURCE):
        raise FileNotFoundError(f"Kernel EFI binary not found at: {EFI_SOURCE}")

    with open(EFI_SOURCE, "rb") as f:
        efi_data = f.read()

    # Also make sure esp/ directory is updated
    os.makedirs(os.path.join(ESP_DIR, "EFI", "BOOT"), exist_ok=True)
    shutil.copyfile(EFI_SOURCE, os.path.join(ESP_DIR, "EFI", "BOOT", "BOOTX64.EFI"))

    startup_nsh = (
        "@echo -off\r\n"
        "echo Starting RUSTNOVA OS...\r\n"
        "if exist fs0:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
        "  fs0:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
        "endif\r\n"
        "if exist fs1:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
        "  fs1:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
        "endif\r\n"
        "\\EFI\\BOOT\\BOOTX64.EFI\r\n"
    ).encode("ascii")

    with open(os.path.join(ESP_DIR, "startup.nsh"), "wb") as f:
        f.write(startup_nsh)

    # Disk Geometry: 64 MB Disk
    SECTOR_SIZE = 512
    TOTAL_SECTORS = 131072                   # 64 MB = 131,072 sectors
    PART_START_LBA = 2048                    # 1 MB boundary
    PART_END_LBA = TOTAL_SECTORS - 34        # 131038
    PART_SECTORS = PART_END_LBA - PART_START_LBA + 1  # 128,991 sectors (~63 MB)

    raw_disk = bytearray(TOTAL_SECTORS * SECTOR_SIZE)

    # 1. LBA 0: Protective MBR
    # MBR partition 1 covers LBA 1 to TOTAL_SECTORS - 1 with type 0xEE
    raw_disk[0x1BE : 0x1CE] = struct.pack(
        "<BBBBBBBBII",
        0x00,               # Non-bootable
        0x00, 0x02, 0x00,   # Start CHS (0, 2, 0)
        0xEE,               # Partition Type: 0xEE (GPT Protective MBR)
        0xFF, 0xFF, 0xFF,   # End CHS
        1,                  # Start LBA: 1
        TOTAL_SECTORS - 1   # Sector count
    )
    raw_disk[510:512] = b"\x55\xAA"

    # 2. Partition Entries (LBA 2 to 33 = 32 sectors = 128 entries of 128 bytes)
    part_entries = bytearray(128 * 128)
    
    # Entry 0: EFI System Partition
    # GUID: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
    esp_type_guid = uuid.UUID("C12A7328-F81F-11D2-BA4B-00A0C93EC93B").bytes_le
    unique_part_guid = uuid.UUID("D3B9D2D7-8025-45C3-8980-A46FBDC16377").bytes_le
    part_name = "EFI System Partition".encode("utf-16le")
    part_name_padded = part_name.ljust(72, b"\x00")

    part_entry_0 = struct.pack(
        "<16s16sQQQ72s",
        esp_type_guid,
        unique_part_guid,
        PART_START_LBA,
        PART_END_LBA,
        0,                  # Attributes
        part_name_padded
    )
    part_entries[0:128] = part_entry_0
    part_entries_crc = zlib.crc32(part_entries)

    # Copy partition entries to LBA 2..33
    raw_disk[2 * SECTOR_SIZE : 34 * SECTOR_SIZE] = part_entries

    # Copy backup partition entries to LBA (TOTAL_SECTORS - 33).. (TOTAL_SECTORS - 2)
    backup_entries_lba = TOTAL_SECTORS - 33
    raw_disk[backup_entries_lba * SECTOR_SIZE : (TOTAL_SECTORS - 1) * SECTOR_SIZE] = part_entries

    # 3. LBA 1: Primary GPT Header (92 bytes)
    disk_guid = uuid.UUID("68B744A6-A228-40F1-8B8E-1D7F2A48C9E1").bytes_le
    def make_gpt_header(my_lba, alt_lba, part_lba):
        hdr = bytearray(92)
        hdr[0:8] = b"EFI PART"
        struct.pack_into("<I", hdr, 8, 0x00010000)   # Revision 1.0
        struct.pack_into("<I", hdr, 12, 92)          # Header size
        struct.pack_into("<I", hdr, 16, 0)           # CRC32 placeholder
        struct.pack_into("<I", hdr, 20, 0)           # Reserved
        struct.pack_into("<Q", hdr, 24, my_lba)      # My LBA
        struct.pack_into("<Q", hdr, 32, alt_lba)     # Alternate LBA
        struct.pack_into("<Q", hdr, 40, PART_START_LBA) # First usable LBA
        struct.pack_into("<Q", hdr, 48, PART_END_LBA)   # Last usable LBA
        hdr[56:72] = disk_guid                       # Disk GUID
        struct.pack_into("<Q", hdr, 72, part_lba)    # Partition entries LBA
        struct.pack_into("<I", hdr, 80, 128)         # Num partition entries
        struct.pack_into("<I", hdr, 84, 128)         # Size of partition entry
        struct.pack_into("<I", hdr, 88, part_entries_crc) # CRC32 of partition array
        crc = zlib.crc32(hdr)
        struct.pack_into("<I", hdr, 16, crc)
        return hdr

    primary_gpt_hdr = make_gpt_header(1, TOTAL_SECTORS - 1, 2)
    raw_disk[1 * SECTOR_SIZE : 1 * SECTOR_SIZE + 92] = primary_gpt_hdr

    # Backup GPT Header at last sector
    backup_gpt_hdr = make_gpt_header(TOTAL_SECTORS - 1, 1, backup_entries_lba)
    raw_disk[(TOTAL_SECTORS - 1) * SECTOR_SIZE : (TOTAL_SECTORS - 1) * SECTOR_SIZE + 92] = backup_gpt_hdr

    # 4. FAT32 Filesystem at PART_START_LBA (LBA 2048)
    part_off = PART_START_LBA * SECTOR_SIZE
    SEC_PER_CLUS = 1          # 512 bytes per cluster
    RSVD_SECTORS = 32         # 32 reserved sectors
    NUM_FATS = 2
    SEC_PER_FAT = 1024        # 1024 sectors = 512 KB per FAT = 131,072 clusters
    ROOT_CLUS = 2

    # FAT32 Boot Sector
    bpb = bytearray(SECTOR_SIZE)
    bpb[0:3] = b"\xEB\x58\x90"     # JMP short 0x58, NOP
    bpb[3:11] = b"MSWIN4.1"        # OEM identifier
    struct.pack_into("<H", bpb, 11, SECTOR_SIZE)
    bpb[13] = SEC_PER_CLUS
    struct.pack_into("<H", bpb, 14, RSVD_SECTORS)
    bpb[16] = NUM_FATS
    struct.pack_into("<H", bpb, 17, 0)             # RootEntCnt = 0 in FAT32
    struct.pack_into("<H", bpb, 19, 0)             # TotSec16 = 0 in FAT32
    bpb[21] = 0xF8                 # Media descriptor
    struct.pack_into("<H", bpb, 22, 0)             # FATSz16 = 0 in FAT32
    struct.pack_into("<H", bpb, 24, 32)            # Sectors per track
    struct.pack_into("<H", bpb, 26, 64)            # Heads
    struct.pack_into("<I", bpb, 28, PART_START_LBA)# Hidden sectors
    struct.pack_into("<I", bpb, 32, PART_SECTORS)  # TotSec32
    struct.pack_into("<I", bpb, 36, SEC_PER_FAT)   # FATSz32
    struct.pack_into("<H", bpb, 40, 0)             # ExtFlags
    struct.pack_into("<H", bpb, 42, 0)             # FSVer
    struct.pack_into("<I", bpb, 44, ROOT_CLUS)     # RootClus (Cluster 2)
    struct.pack_into("<H", bpb, 48, 1)             # FSInfo sector
    struct.pack_into("<H", bpb, 50, 6)             # BkBootSec
    bpb[64] = 0x80                 # DrvNum
    bpb[65] = 0x00                 # Reserved
    bpb[66] = 0x29                 # BootSig
    struct.pack_into("<I", bpb, 67, 0x524E4F53)    # VolID ("RNOS")
    bpb[71:82] = b"RUSTNOVA   "    # VolLab (11 chars)
    bpb[82:90] = b"FAT32   "       # FilSysType (8 chars)
    bpb[510:512] = b"\x55\xAA"

    raw_disk[part_off : part_off + SECTOR_SIZE] = bpb
    # Backup boot sector at sector 6
    raw_disk[part_off + 6 * SECTOR_SIZE : part_off + 7 * SECTOR_SIZE] = bpb

    # FSInfo Sector (Sector 1)
    fsinfo = bytearray(SECTOR_SIZE)
    fsinfo[0:4] = b"RRaA"          # Lead signature
    fsinfo[484:488] = b"rrAa"      # Structure signature
    struct.pack_into("<I", fsinfo, 488, 126000)    # Free cluster count
    struct.pack_into("<I", fsinfo, 492, 7)         # Next free cluster
    fsinfo[510:512] = b"\x55\xAA"
    raw_disk[part_off + 1 * SECTOR_SIZE : part_off + 2 * SECTOR_SIZE] = fsinfo

    # 5. Populate FAT Tables & Clusters
    fat1_off = part_off + (RSVD_SECTORS * SECTOR_SIZE)
    fat2_off = fat1_off + (SEC_PER_FAT * SECTOR_SIZE)
    data_off = fat2_off + (SEC_PER_FAT * SECTOR_SIZE)
    CLUS_SIZE = SEC_PER_CLUS * SECTOR_SIZE          # 512 bytes

    def cluster_offset(c):
        return data_off + (c - 2) * CLUS_SIZE

    def make_dir_entry(name8, ext3, attr, first_cluster, file_size):
        name_field = f"{name8:<8}{ext3:<3}".encode("ascii")
        return struct.pack(
            "<11sBBBHHHHHHHI",
            name_field,
            attr,
            0,                     # NT reserved
            0,                     # Creation time 10ms
            0x4800,                # Creation time (09:00:00)
            0x5D77,                # Creation date (2026-11-23)
            0x5D77,                # Last access date
            (first_cluster >> 16) & 0xFFFF, # High 16 bits of cluster in FAT32
            0x4800,                # Write time
            0x5D77,                # Write date
            first_cluster & 0xFFFF,# Low 16 bits of cluster
            file_size              # File size
        )

    # Clusters:
    # Cluster 2: Root Directory
    # Cluster 3: \EFI directory
    # Cluster 4: \startup.nsh
    # Cluster 5: \EFI\BOOT directory
    # Cluster 6..6+N: \EFI\BOOT\BOOTX64.EFI
    CLUS_ROOT = 2
    CLUS_EFI_DIR = 3
    CLUS_STARTUP_NSH = 4
    CLUS_BOOT_DIR = 5
    CLUS_BOOTX64 = 6

    # Cluster 2: Root Directory
    root_data = bytearray(CLUS_SIZE)
    root_data[0:32] = make_dir_entry("RUSTNOVA", "   ", 0x08, 0, 0)
    root_data[32:64] = make_dir_entry("EFI     ", "   ", 0x10, CLUS_EFI_DIR, 0)
    root_data[64:96] = make_dir_entry("STARTUP ", "NSH", 0x20, CLUS_STARTUP_NSH, len(startup_nsh))
    raw_disk[cluster_offset(CLUS_ROOT) : cluster_offset(CLUS_ROOT) + CLUS_SIZE] = root_data

    # Cluster 3: \EFI directory
    efi_dir = bytearray(CLUS_SIZE)
    efi_dir[0:32] = make_dir_entry(".       ", "   ", 0x10, CLUS_EFI_DIR, 0)
    efi_dir[32:64] = make_dir_entry("..      ", "   ", 0x10, 0, 0)
    efi_dir[64:96] = make_dir_entry("BOOT    ", "   ", 0x10, CLUS_BOOT_DIR, 0)
    raw_disk[cluster_offset(CLUS_EFI_DIR) : cluster_offset(CLUS_EFI_DIR) + CLUS_SIZE] = efi_dir

    # Cluster 4: \startup.nsh
    c4_off = cluster_offset(CLUS_STARTUP_NSH)
    raw_disk[c4_off : c4_off + len(startup_nsh)] = startup_nsh

    # Cluster 5: \EFI\BOOT directory
    boot_dir = bytearray(CLUS_SIZE)
    boot_dir[0:32] = make_dir_entry(".       ", "   ", 0x10, CLUS_BOOT_DIR, 0)
    boot_dir[32:64] = make_dir_entry("..      ", "   ", 0x10, CLUS_EFI_DIR, 0)
    boot_dir[64:96] = make_dir_entry("BOOTX64 ", "EFI", 0x20, CLUS_BOOTX64, len(efi_data))
    raw_disk[cluster_offset(CLUS_BOOT_DIR) : cluster_offset(CLUS_BOOT_DIR) + CLUS_SIZE] = boot_dir

    # Clusters 6+: \EFI\BOOT\BOOTX64.EFI
    efi_clusters_needed = (len(efi_data) + CLUS_SIZE - 1) // CLUS_SIZE
    c6_off = cluster_offset(CLUS_BOOTX64)
    raw_disk[c6_off : c6_off + len(efi_data)] = efi_data

    # Populate FAT Table (32-bit entries)
    fat_entries = [0] * (SEC_PER_FAT * SECTOR_SIZE // 4)
    fat_entries[0] = 0x0FFFFFF8
    fat_entries[1] = 0x0FFFFFFF
    fat_entries[CLUS_ROOT] = 0x0FFFFFFF
    fat_entries[CLUS_EFI_DIR] = 0x0FFFFFFF
    fat_entries[CLUS_STARTUP_NSH] = 0x0FFFFFFF
    fat_entries[CLUS_BOOT_DIR] = 0x0FFFFFFF

    for i in range(efi_clusters_needed):
        c = CLUS_BOOTX64 + i
        if i == efi_clusters_needed - 1:
            fat_entries[c] = 0x0FFFFFFF
        else:
            fat_entries[c] = c + 1

    fat_bytes = struct.pack(f"<{len(fat_entries)}I", *fat_entries)
    raw_disk[fat1_off : fat1_off + len(fat_bytes)] = fat_bytes
    raw_disk[fat2_off : fat2_off + len(fat_bytes)] = fat_bytes

    # Write out raw disk image
    with open(RAW_OUTPUT, "wb") as f:
        f.write(raw_disk)

    print(f"[OK] Generated standard 64 MB GPT FAT32 ESP raw disk: {RAW_OUTPUT} ({len(raw_disk)} bytes)")

    # Convert to VirtualBox VDI (.vdi)
    if os.path.exists(VDI_OUTPUT):
        os.remove(VDI_OUTPUT)
    subprocess.run([QEMU_IMG, "convert", "-f", "raw", "-O", "vdi", RAW_OUTPUT, VDI_OUTPUT], check=True)
    print(f"[OK] Generated VirtualBox VDI: {VDI_OUTPUT} ({os.path.getsize(VDI_OUTPUT)} bytes)")

    # Convert to VMDK (.vmdk)
    if os.path.exists(VMDK_OUTPUT):
        os.remove(VMDK_OUTPUT)
    subprocess.run([QEMU_IMG, "convert", "-f", "raw", "-O", "vmdk", RAW_OUTPUT, VMDK_OUTPUT], check=True)
    print(f"[OK] Generated VMDK: {VMDK_OUTPUT} ({os.path.getsize(VMDK_OUTPUT)} bytes)")

if __name__ == "__main__":
    build_gpt_fat32_disk()
