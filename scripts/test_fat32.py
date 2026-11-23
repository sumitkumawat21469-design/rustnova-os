#!/usr/bin/env python3
import struct
import zlib
import subprocess
import os
import shutil

BASE_DIR = r"C:\Users\admin\.gemini\antigravity\scratch\rustnova-os"
EFI_SOURCE = os.path.join(BASE_DIR, "esp", "EFI", "BOOT", "BOOTX64.EFI")
TEST_RAW = os.path.join(BASE_DIR, "test_fat32.raw")
TEST_VDI = os.path.join(BASE_DIR, "test_fat32.vdi")
QEMU_IMG = r"C:\Users\admin\.gemini\antigravity\scratch\tools\qemu\qemu-img.exe"

with open(EFI_SOURCE, "rb") as f:
    efi_data = f.read()

startup_nsh = (
    "@echo -off\r\n"
    "echo Booting RUSTNOVA OS...\r\n"
    "if exist fs0:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
    "  fs0:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
    "endif\r\n"
    "if exist fs1:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
    "  fs1:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
    "endif\r\n"
    "\\EFI\\BOOT\\BOOTX64.EFI\r\n"
).encode("ascii")

# 64 MB disk = 131,072 sectors
TOTAL_SECTORS = 131072
SECTOR_SIZE = 512
PART_START = 2048
PART_SECTORS = TOTAL_SECTORS - PART_START # 129024 sectors (~63 MB)

disk = bytearray(TOTAL_SECTORS * SECTOR_SIZE)

# MBR at LBA 0:
# Partition 1: Type 0xEF (EFI System Partition), Start 2048, Length 129024
disk[0x1BE : 0x1CE] = struct.pack(
    "<BBBBBBBBII",
    0x80,               # Active / Bootable
    0x00, 0x01, 0x01,   # Start CHS
    0xEF,               # Partition Type: 0xEF (EFI System Partition)
    0xFE, 0xFF, 0xFF,   # End CHS
    PART_START,
    PART_SECTORS
)
disk[510:512] = b"\x55\xAA"

# FAT32 Partition at PART_START (LBA 2048)
part_off = PART_START * SECTOR_SIZE
SEC_PER_CLUS = 1        # 512 bytes per cluster
RSVD_SECTORS = 32
NUM_FATS = 2
SEC_PER_FAT = 1024      # 1024 sectors = 512 KB per FAT
ROOT_CLUS = 2

# BPB
bpb = bytearray(SECTOR_SIZE)
bpb[0:3] = b"\xEB\x58\x90"
bpb[3:11] = b"MSWIN4.1"
struct.pack_into("<H", bpb, 11, SECTOR_SIZE)
bpb[13] = SEC_PER_CLUS
struct.pack_into("<H", bpb, 14, RSVD_SECTORS)
bpb[16] = NUM_FATS
struct.pack_into("<H", bpb, 17, 0)         # RootEntCnt = 0 in FAT32
struct.pack_into("<H", bpb, 19, 0)         # TotSec16 = 0 in FAT32
bpb[21] = 0xF8             # Media descriptor
struct.pack_into("<H", bpb, 22, 0)         # FATSz16 = 0 in FAT32
struct.pack_into("<H", bpb, 24, 32)        # SecPerTrk
struct.pack_into("<H", bpb, 26, 64)        # Heads
struct.pack_into("<I", bpb, 28, PART_START)# HiddSec
struct.pack_into("<I", bpb, 32, PART_SECTORS)# TotSec32
struct.pack_into("<I", bpb, 36, SEC_PER_FAT)# FATSz32
struct.pack_into("<H", bpb, 40, 0)         # ExtFlags
struct.pack_into("<H", bpb, 42, 0)         # FSVer
struct.pack_into("<I", bpb, 44, ROOT_CLUS) # RootClus
struct.pack_into("<H", bpb, 48, 1)         # FSInfo
struct.pack_into("<H", bpb, 50, 6)         # BkBootSec
bpb[64] = 0x80             # DrvNum
bpb[66] = 0x29             # BootSig
struct.pack_into("<I", bpb, 67, 0x524E4F53)# VolID
bpb[71:82] = b"RUSTNOVA   "
bpb[82:90] = b"FAT32   "
bpb[510:512] = b"\x55\xAA"

disk[part_off : part_off + SECTOR_SIZE] = bpb
disk[part_off + 6 * SECTOR_SIZE : part_off + 7 * SECTOR_SIZE] = bpb

# FSInfo sector at LBA 2048 + 1
fsinfo = bytearray(SECTOR_SIZE)
fsinfo[0:4] = b"RRaA"
fsinfo[484:488] = b"rrAa"
struct.pack_into("<I", fsinfo, 488, 126000)
struct.pack_into("<I", fsinfo, 492, 7)
fsinfo[510:512] = b"\x55\xAA"
disk[part_off + 1 * SECTOR_SIZE : part_off + 2 * SECTOR_SIZE] = fsinfo

# Directory and data layout
fat1_off = part_off + (RSVD_SECTORS * SECTOR_SIZE)
fat2_off = fat1_off + (SEC_PER_FAT * SECTOR_SIZE)
data_off = fat2_off + (SEC_PER_FAT * SECTOR_SIZE)
CLUS_SIZE = SEC_PER_CLUS * SECTOR_SIZE

def clus_offset(c):
    return data_off + (c - 2) * CLUS_SIZE

def make_entry(name8, ext3, attr, first_c, size):
    name_field = f"{name8:<8}{ext3:<3}".encode("ascii")
    return struct.pack(
        "<11sBBBHHHHHHHI",
        name_field, attr, 0, 0, 0x4800, 0x5D77, 0x5D77,
        (first_c >> 16) & 0xFFFF,
        0x4800, 0x5D77,
        first_c & 0xFFFF,
        size
    )

# Cluster 2: Root Directory
# Cluster 3: \EFI
# Cluster 4: \startup.nsh
# Cluster 5: \EFI\BOOT
# Cluster 6+: \EFI\BOOT\BOOTX64.EFI
root_data = bytearray(CLUS_SIZE)
root_data[0:32] = make_entry("RUSTNOVA", "   ", 0x08, 0, 0)
root_data[32:64] = make_entry("EFI     ", "   ", 0x10, 3, 0)
root_data[64:96] = make_entry("STARTUP ", "NSH", 0x20, 4, len(startup_nsh))
disk[clus_offset(2) : clus_offset(2) + CLUS_SIZE] = root_data

# Cluster 3: \EFI
efi_data_dir = bytearray(CLUS_SIZE)
efi_data_dir[0:32] = make_entry(".       ", "   ", 0x10, 3, 0)
efi_data_dir[32:64] = make_entry("..      ", "   ", 0x10, 0, 0)
efi_data_dir[64:96] = make_entry("BOOT    ", "   ", 0x10, 5, 0)
disk[clus_offset(3) : clus_offset(3) + CLUS_SIZE] = efi_data_dir

# Cluster 4: \startup.nsh
disk[clus_offset(4) : clus_offset(4) + len(startup_nsh)] = startup_nsh

# Cluster 5: \EFI\BOOT
boot_data_dir = bytearray(CLUS_SIZE)
boot_data_dir[0:32] = make_entry(".       ", "   ", 0x10, 5, 0)
boot_data_dir[32:64] = make_entry("..      ", "   ", 0x10, 3, 0)
boot_data_dir[64:96] = make_entry("BOOTX64 ", "EFI", 0x20, 6, len(efi_data))
disk[clus_offset(5) : clus_offset(5) + CLUS_SIZE] = boot_data_dir

# Cluster 6+: \EFI\BOOT\BOOTX64.EFI
efi_clus_count = (len(efi_data) + CLUS_SIZE - 1) // CLUS_SIZE
disk[clus_offset(6) : clus_offset(6) + len(efi_data)] = efi_data

# FAT entries (32-bit)
fat = [0] * (SEC_PER_FAT * SECTOR_SIZE // 4)
fat[0] = 0x0FFFFFF8
fat[1] = 0x0FFFFFFF
fat[2] = 0x0FFFFFFF # Root dir EOF
fat[3] = 0x0FFFFFFF # \EFI dir EOF
fat[4] = 0x0FFFFFFF # startup.nsh EOF
fat[5] = 0x0FFFFFFF # \EFI\BOOT dir EOF

for i in range(efi_clus_count):
    c = 6 + i
    if i == efi_clus_count - 1:
        fat[c] = 0x0FFFFFFF
    else:
        fat[c] = c + 1

fat_bytes = struct.pack(f"<{len(fat)}I", *fat)
disk[fat1_off : fat1_off + len(fat_bytes)] = fat_bytes
disk[fat2_off : fat2_off + len(fat_bytes)] = fat_bytes

with open(TEST_RAW, "wb") as f:
    f.write(disk)

if os.path.exists(TEST_VDI): os.remove(TEST_VDI)
subprocess.run([QEMU_IMG, "convert", "-f", "raw", "-O", "vdi", TEST_RAW, TEST_VDI], check=True)
print(f"Generated {TEST_VDI} ({os.path.getsize(TEST_VDI)} bytes)")
