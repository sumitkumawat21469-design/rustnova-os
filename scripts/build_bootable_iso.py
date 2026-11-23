#!/usr/bin/env python3
"""
RustNova OS - Standard UEFI Bootable ISO Generator (ISO-9660 + El Torito UEFI + Hybrid MBR).

Generates a fully compliant bootable ISO image (rustnova-os.iso) containing:
  - ISO-9660 filesystem with Primary Volume Descriptor and Terminator
  - El Torito Boot Record and Boot Catalog with Platform ID 0xEF (UEFI)
  - Embedded FAT16 EFI System Partition image with \\EFI\\BOOT\\BOOTX64.EFI and \\startup.nsh
  - Hybrid MBR at Sector 0 for dual CD/DVD optical drive and USB flash drive bootability.

Compatible with VirtualBox 7.x UEFI, QEMU OVMF, VMware, and physical hardware.
"""

import os
import struct
import shutil

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
EFI_SOURCE = os.path.join(BASE_DIR, "esp", "EFI", "BOOT", "BOOTX64.EFI")
ISO_OUTPUT = os.path.join(BASE_DIR, "rustnova-os.iso")

def build_fat16_esp_image(efi_binary_data, startup_nsh_data):
    """
    Creates a 4 MB FAT16 filesystem image (8,192 sectors of 512 bytes)
    containing \\startup.nsh and \\EFI\\BOOT\\BOOTX64.EFI.
    """
    TOTAL_SECTORS = 8192
    SECTOR_SIZE = 512
    SEC_PER_CLUS = 1
    RSVD_SECTORS = 1
    NUM_FATS = 2
    ROOT_ENT_CNT = 512
    SEC_PER_FAT = 64
    ROOT_DIR_SECTORS = (ROOT_ENT_CNT * 32 + SECTOR_SIZE - 1) // SECTOR_SIZE # 32 sectors

    img = bytearray(TOTAL_SECTORS * SECTOR_SIZE)

    # 1. FAT16 Boot Sector (BPB) at LBA 0
    bpb = bytearray(SECTOR_SIZE)
    bpb[0:3] = b"\xEB\x3C\x90"
    bpb[3:11] = b"MSDOS5.0"
    struct.pack_into("<H", bpb, 11, SECTOR_SIZE)
    bpb[13] = SEC_PER_CLUS
    struct.pack_into("<H", bpb, 14, RSVD_SECTORS)
    bpb[16] = NUM_FATS
    struct.pack_into("<H", bpb, 17, ROOT_ENT_CNT)
    struct.pack_into("<H", bpb, 19, TOTAL_SECTORS) # TotSec16
    bpb[21] = 0xF8 # Media descriptor
    struct.pack_into("<H", bpb, 22, SEC_PER_FAT)
    struct.pack_into("<H", bpb, 24, 32) # SecPerTrk
    struct.pack_into("<H", bpb, 26, 64) # Heads
    struct.pack_into("<I", bpb, 28, 0)  # HiddSec
    struct.pack_into("<I", bpb, 32, 0)  # TotSec32
    bpb[36] = 0x80 # DrvNum
    bpb[38] = 0x29 # BootSig
    struct.pack_into("<I", bpb, 39, 0x524E4F53) # VolID ("RNOS")
    bpb[43:54] = b"RUSTNOVA   "
    bpb[54:62] = b"FAT16   "
    bpb[510:512] = b"\x55\xAA"
    img[0:SECTOR_SIZE] = bpb

    # Layout offsets
    fat1_off = RSVD_SECTORS * SECTOR_SIZE
    fat2_off = fat1_off + (SEC_PER_FAT * SECTOR_SIZE)
    root_off = fat2_off + (SEC_PER_FAT * SECTOR_SIZE)
    data_off = root_off + (ROOT_DIR_SECTORS * SECTOR_SIZE)
    CLUS_SIZE = SEC_PER_CLUS * SECTOR_SIZE

    def clus_offset(c):
        return data_off + (c - 2) * CLUS_SIZE

    def make_entry(name8, ext3, attr, first_c, size):
        name_field = f"{name8:<8}{ext3:<3}".encode("ascii")
        return struct.pack(
            "<11sBBBHHHHHHHI",
            name_field, attr, 0, 0, 0x4800, 0x5D77, 0x5D77,
            0, # High 16 bits of cluster (0 in FAT16)
            0x4800, 0x5D77,
            first_c & 0xFFFF,
            size
        )

    # Root Directory
    img[root_off : root_off + 32] = make_entry("RUSTNOVA", "   ", 0x08, 0, 0)
    img[root_off + 32 : root_off + 64] = make_entry("EFI     ", "   ", 0x10, 2, 0)
    img[root_off + 64 : root_off + 96] = make_entry("STARTUP ", "NSH", 0x20, 3, len(startup_nsh_data))

    # Cluster 2: \EFI directory
    efi_dir = bytearray(CLUS_SIZE)
    efi_dir[0:32] = make_entry(".       ", "   ", 0x10, 2, 0)
    efi_dir[32:64] = make_entry("..      ", "   ", 0x10, 0, 0)
    efi_dir[64:96] = make_entry("BOOT    ", "   ", 0x10, 4, 0)
    img[clus_offset(2) : clus_offset(2) + CLUS_SIZE] = efi_dir

    # Cluster 3: \startup.nsh
    img[clus_offset(3) : clus_offset(3) + len(startup_nsh_data)] = startup_nsh_data

    # Cluster 4: \EFI\BOOT directory
    boot_dir = bytearray(CLUS_SIZE)
    boot_dir[0:32] = make_entry(".       ", "   ", 0x10, 4, 0)
    boot_dir[32:64] = make_entry("..      ", "   ", 0x10, 2, 0)
    boot_dir[64:96] = make_entry("BOOTX64 ", "EFI", 0x20, 5, len(efi_binary_data))
    img[clus_offset(4) : clus_offset(4) + CLUS_SIZE] = boot_dir

    # Cluster 5+: \EFI\BOOT\BOOTX64.EFI
    efi_clus_count = (len(efi_binary_data) + CLUS_SIZE - 1) // CLUS_SIZE
    img[clus_offset(5) : clus_offset(5) + len(efi_binary_data)] = efi_binary_data

    # FAT Tables (16-bit entries)
    fat = [0] * (SEC_PER_FAT * SECTOR_SIZE // 2)
    fat[0] = 0xFFF8
    fat[1] = 0xFFFF
    fat[2] = 0xFFFF # \EFI dir EOF
    fat[3] = 0xFFFF # startup.nsh EOF
    fat[4] = 0xFFFF # \EFI\BOOT dir EOF

    for i in range(efi_clus_count):
        c = 5 + i
        if i == efi_clus_count - 1:
            fat[c] = 0xFFFF
        else:
            fat[c] = c + 1

    fat_bytes = struct.pack(f"<{len(fat)}H", *fat)
    img[fat1_off : fat1_off + len(fat_bytes)] = fat_bytes
    img[fat2_off : fat2_off + len(fat_bytes)] = fat_bytes

    return bytes(img)

def make_iso_dir_record(lba, size, name_bytes, is_dir=False):
    name_len = len(name_bytes)
    rec_len = 33 + name_len
    if rec_len % 2 != 0:
        rec_len += 1
    rec = bytearray(rec_len)
    rec[0] = rec_len
    rec[1] = 0
    struct.pack_into("<I", rec, 2, lba)
    struct.pack_into(">I", rec, 6, lba)
    struct.pack_into("<I", rec, 10, size)
    struct.pack_into(">I", rec, 14, size)
    rec[18:25] = bytes([126, 11, 24, 0, 0, 0, 0]) # Date
    rec[25] = 0x02 if is_dir else 0x00
    rec[26] = 0
    rec[27] = 0
    struct.pack_into("<H", rec, 28, 1)
    struct.pack_into(">H", rec, 30, 1)
    rec[32] = name_len
    rec[33:33+name_len] = name_bytes
    return rec

def build_bootable_iso():
    if not os.path.exists(EFI_SOURCE):
        alt = os.path.join(BASE_DIR, "target", "x86_64-unknown-uefi", "debug", "rustnova-os.efi")
        if os.path.exists(alt):
            os.makedirs(os.path.dirname(EFI_SOURCE), exist_ok=True)
            shutil.copyfile(alt, EFI_SOURCE)
        else:
            raise FileNotFoundError(f"Kernel EFI binary not found: {EFI_SOURCE}")

    with open(EFI_SOURCE, "rb") as f:
        efi_data = f.read()

    startup_nsh = (
        "@echo -off\r\n"
        "echo Booting RUSTNOVA OS (UEFI ISO)...\r\n"
        "if exist fs0:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
        "  fs0:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
        "endif\r\n"
        "if exist fs1:\\EFI\\BOOT\\BOOTX64.EFI then\r\n"
        "  fs1:\\EFI\\BOOT\\BOOTX64.EFI\r\n"
        "endif\r\n"
        "\\EFI\\BOOT\\BOOTX64.EFI\r\n"
    ).encode("ascii")

    # Generate 4 MB FAT16 EFI System Partition image
    efi_fat_img = build_fat16_esp_image(efi_data, startup_nsh)
    efi_fat_len = len(efi_fat_img)
    efi_fat_sectors_512 = efi_fat_len // 512
    efi_fat_sectors_2048 = (efi_fat_len + 2047) // 2048

    # Layout of 2048-byte ISO Sectors:
    # Sector 0..15:   System Area (with MBR at Sector 0)
    # Sector 16:       Primary Volume Descriptor (PVD)
    # Sector 17:       Boot Record Volume Descriptor (BRVD - El Torito)
    # Sector 18:       Volume Descriptor Set Terminator (VDST)
    # Sector 19:       Boot Catalog (El Torito)
    # Sector 20:       Root Directory Record
    # Sector 21..end:  EFI FAT System Partition Image
    LBA_PVD = 16
    LBA_BRVD = 17
    LBA_VDST = 18
    LBA_BOOT_CAT = 19
    LBA_ROOT_DIR = 20
    LBA_EFI_IMG = 21

    TOTAL_ISO_SECTORS = LBA_EFI_IMG + efi_fat_sectors_2048

    iso = bytearray(TOTAL_ISO_SECTORS * 2048)

    # 1. Hybrid MBR at Sector 0 (LBA 0 of ISO)
    # Partition 1: Type 0xEF (EFI System Partition), starts at LBA_EFI_IMG * 4 (512-byte sectors)
    mbr_start_512 = LBA_EFI_IMG * 4
    iso[0x1BE : 0x1CE] = struct.pack(
        "<BBBBBBBBII",
        0x80,               # Active / Bootable flag
        0x00, 0x01, 0x01,   # Start CHS
        0xEF,               # Partition Type: 0xEF (EFI System Partition)
        0xFE, 0xFF, 0xFF,   # End CHS
        mbr_start_512,
        efi_fat_sectors_512
    )
    iso[510:512] = b"\x55\xAA"

    # 2. Primary Volume Descriptor at Sector 16
    pvd = bytearray(2048)
    pvd[0] = 0x01                          # Primary Volume Descriptor
    pvd[1:6] = b"CD001"
    pvd[6] = 0x01
    pvd[8:40] = f"{'RUSTNOVA_OS':<32}".encode("ascii")
    pvd[40:72] = f"{'RUSTNOVA_OS':<32}".encode("ascii")
    struct.pack_into("<I", pvd, 80, TOTAL_ISO_SECTORS)
    struct.pack_into(">I", pvd, 84, TOTAL_ISO_SECTORS)
    struct.pack_into("<H", pvd, 120, 1)    # Volume Set Size
    struct.pack_into(">H", pvd, 122, 1)
    struct.pack_into("<H", pvd, 124, 1)    # Volume Sequence Number
    struct.pack_into(">H", pvd, 126, 1)
    struct.pack_into("<H", pvd, 128, 2048) # Logical Block Size
    struct.pack_into(">H", pvd, 130, 2048)
    struct.pack_into("<I", pvd, 132, 10)   # Path Table Size
    struct.pack_into(">I", pvd, 136, 10)
    # Root directory record in PVD (offset 156)
    pvd[156:190] = make_iso_dir_record(LBA_ROOT_DIR, 2048, b"\x00", is_dir=True)
    pvd[190:318] = f"{'RUSTNOVA_OS':<128}".encode("ascii")
    pvd[318:446] = f"{'RUSTNOVA_OS':<128}".encode("ascii")
    pvd[446:574] = f"{'RUSTNOVA_OS':<128}".encode("ascii")
    pvd[574:702] = f"{'RUSTNOVA_OS':<128}".encode("ascii")
    date_str = b"2026112400000000\x00"
    pvd[813:830] = date_str
    pvd[830:847] = date_str
    pvd[847:864] = date_str
    pvd[864:881] = date_str
    pvd[881] = 0x01
    iso[LBA_PVD * 2048 : (LBA_PVD + 1) * 2048] = pvd

    # 3. Boot Record Volume Descriptor (El Torito) at Sector 17
    brvd = bytearray(2048)
    brvd[0] = 0x00                         # Boot Record
    brvd[1:6] = b"CD001"
    brvd[6] = 0x01
    brvd[7:39] = b"EL TORITO SPECIFICATION" + b"\x00" * 9
    struct.pack_into("<I", brvd, 71, LBA_BOOT_CAT) # Boot Catalog LBA
    iso[LBA_BRVD * 2048 : (LBA_BRVD + 1) * 2048] = brvd

    # 4. Volume Descriptor Set Terminator at Sector 18
    vdst = bytearray(2048)
    vdst[0] = 0xFF
    vdst[1:6] = b"CD001"
    vdst[6] = 0x01
    iso[LBA_VDST * 2048 : (LBA_VDST + 1) * 2048] = vdst

    # 5. Boot Catalog (El Torito) at Sector 19
    cat = bytearray(2048)
    # Validation Entry (32 bytes at 0)
    cat[0] = 0x01
    cat[1] = 0x00
    cat[4:15] = b"RUSTNOVA OS"
    cat[30] = 0x55
    cat[31] = 0xAA
    words = struct.unpack("<16H", cat[0:32])
    csum = (-sum(words)) & 0xFFFF
    struct.pack_into("<H", cat, 28, csum)

    # Initial / Default Entry (BIOS) at 32
    cat[32] = 0x00                         # Not bootable
    cat[33] = 0x00                         # No emulation
    struct.pack_into("<H", cat, 38, 4)     # 4 512-byte sectors
    struct.pack_into("<I", cat, 40, LBA_EFI_IMG)

    # Section Header Entry for EFI at 64
    cat[64] = 0x91                         # Final section header
    cat[65] = 0xEF                         # Platform ID: EFI
    struct.pack_into("<H", cat, 66, 1)     # 1 section entry following
    cat[68:93] = b"UEFI EFI SYSTEM PARTITION"

    # Section Entry for EFI at 96
    cat[96] = 0x88                         # 0x88: Bootable!
    cat[97] = 0x00                         # No emulation
    cat[100] = 0xEF                        # System type: EFI
    struct.pack_into("<H", cat, 102, efi_fat_sectors_512)
    struct.pack_into("<I", cat, 104, LBA_EFI_IMG)

    iso[LBA_BOOT_CAT * 2048 : (LBA_BOOT_CAT + 1) * 2048] = cat

    # 6. Root Directory at Sector 20
    root_dir = bytearray(2048)
    pos = 0
    # Entry .
    dot = make_iso_dir_record(LBA_ROOT_DIR, 2048, b"\x00", is_dir=True)
    root_dir[pos : pos + len(dot)] = dot
    pos += len(dot)
    # Entry ..
    dotdot = make_iso_dir_record(LBA_ROOT_DIR, 2048, b"\x01", is_dir=True)
    root_dir[pos : pos + len(dotdot)] = dotdot
    pos += len(dotdot)
    # Entry BOOT.CAT;1
    boot_cat_entry = make_iso_dir_record(LBA_BOOT_CAT, 2048, b"BOOT.CAT;1", is_dir=False)
    root_dir[pos : pos + len(boot_cat_entry)] = boot_cat_entry
    pos += len(boot_cat_entry)
    # Entry EFI.IMG;1
    efi_img_entry = make_iso_dir_record(LBA_EFI_IMG, efi_fat_len, b"EFI.IMG;1", is_dir=False)
    root_dir[pos : pos + len(efi_img_entry)] = efi_img_entry
    pos += len(efi_img_entry)

    iso[LBA_ROOT_DIR * 2048 : (LBA_ROOT_DIR + 1) * 2048] = root_dir

    # 7. EFI System Partition image at Sector 21+
    iso[LBA_EFI_IMG * 2048 : LBA_EFI_IMG * 2048 + efi_fat_len] = efi_fat_img

    with open(ISO_OUTPUT, "wb") as f:
        f.write(iso)

    size = os.path.getsize(ISO_OUTPUT)
    print(f"[OK] Generated UEFI Bootable ISO: {ISO_OUTPUT} ({size} bytes, {size / (1024*1024):.2f} MB)")
    return ISO_OUTPUT

if __name__ == "__main__":
    build_bootable_iso()
