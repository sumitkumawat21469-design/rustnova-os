//! UEFI Protocol Definitions and Structures.
//!
//! Provides bare-metal definitions for UEFI System Table, Boot Services,
//! Memory Map Descriptors, and Graphics Output Protocol (GOP).

#![allow(non_snake_case, non_camel_case_types, dead_code)]

pub type EfiHandle = *mut ();
pub type EfiStatus = usize;

pub const EFI_SUCCESS: EfiStatus = 0;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct EfiGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl EfiGuid {
    pub const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        Self { data1, data2, data3, data4 }
    }
}

/// EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID
/// {9042a9de-23dc-4a38-96fb-7aded080516a}
pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid::new(
    0x9042a9de,
    0x23dc,
    0x4a38,
    [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
);

#[repr(C)]
pub struct EfiTableHeader {
    pub Signature: u64,
    pub Revision: u32,
    pub HeaderSize: u32,
    pub CRC32: u32,
    pub Reserved: u32,
}

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    pub Reset: usize,
    pub OutputString: unsafe extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, string: *const u16) -> EfiStatus,
    pub TestString: usize,
    pub QueryMode: usize,
    pub SetMode: usize,
    pub SetAttribute: usize,
    pub ClearScreen: unsafe extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol) -> EfiStatus,
    pub SetCursorPosition: usize,
    pub EnableCursor: usize,
    pub Mode: *mut (),
}

#[repr(C)]
pub struct EfiSystemTable {
    pub Hdr: EfiTableHeader,
    pub FirmwareVendor: *const u16,
    pub FirmwareRevision: u32,
    pub ConsoleInHandle: EfiHandle,
    pub ConIn: *mut (),
    pub ConsoleOutHandle: EfiHandle,
    pub ConOut: *mut EfiSimpleTextOutputProtocol,
    pub StandardErrorHandle: EfiHandle,
    pub StdErr: *mut EfiSimpleTextOutputProtocol,
    pub RuntimeServices: *mut (),
    pub BootServices: *mut EfiBootServices,
    pub NumberOfTableEntries: usize,
    pub ConfigurationTable: *mut (),
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum EfiGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor = 0,
    PixelBlueGreenRedReserved8BitPerColor = 1,
    PixelBitMask = 2,
    PixelBltOnly = 3,
    PixelFormatMax = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EfiGraphicsOutputModeInformation {
    pub Version: u32,
    pub HorizontalResolution: u32,
    pub VerticalResolution: u32,
    pub PixelFormat: EfiGraphicsPixelFormat,
    pub PixelInformation: [u32; 4],
    pub PixelsPerScanLine: u32,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocolMode {
    pub MaxMode: u32,
    pub Mode: u32,
    pub Info: *mut EfiGraphicsOutputModeInformation,
    pub SizeOfInfo: usize,
    pub FrameBufferBase: u64,
    pub FrameBufferSize: usize,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub QueryMode: usize,
    pub SetMode: usize,
    pub Blt: usize,
    pub Mode: *mut EfiGraphicsOutputProtocolMode,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EfiMemoryDescriptor {
    pub Type: u32,
    pub PhysicalStart: u64,
    pub VirtualStart: u64,
    pub NumberOfPages: u64,
    pub Attribute: u64,
}

#[repr(C)]
pub struct EfiBootServices {
    pub Hdr: EfiTableHeader,
    pub RaiseTPL: usize,
    pub RestoreTPL: usize,
    pub AllocatePages: usize,
    pub FreePages: usize,
    pub GetMemoryMap: unsafe extern "efiapi" fn(
        MemoryMapSize: *mut usize,
        MemoryMap: *mut EfiMemoryDescriptor,
        MapKey: *mut usize,
        DescriptorSize: *mut usize,
        DescriptorVersion: *mut u32,
    ) -> EfiStatus,
    pub AllocatePool: usize,
    pub FreePool: usize,
    pub CreateEvent: usize,
    pub SetTimer: usize,
    pub WaitForEvent: usize,
    pub SignalEvent: usize,
    pub CloseEvent: usize,
    pub CheckEvent: usize,
    pub InstallProtocolInterface: usize,
    pub ReinstallProtocolInterface: usize,
    pub UninstallProtocolInterface: usize,
    pub HandleProtocol: unsafe extern "efiapi" fn(
        Handle: EfiHandle,
        Protocol: *const EfiGuid,
        Interface: *mut *mut (),
    ) -> EfiStatus,
    pub VoidReserved: *mut (),
    pub RegisterProtocolNotify: usize,
    pub LocateHandle: usize,
    pub LocateDevicePath: usize,
    pub InstallConfigurationTable: usize,
    pub LoadImage: usize,
    pub StartImage: usize,
    pub Exit: usize,
    pub UnloadImage: usize,
    pub ExitBootServices: unsafe extern "efiapi" fn(ImageHandle: EfiHandle, MapKey: usize) -> EfiStatus,
    pub GetNextMonotonicCount: usize,
    pub Stall: unsafe extern "efiapi" fn(Microseconds: usize) -> EfiStatus,
    pub SetWatchdogTimer: unsafe extern "efiapi" fn(Timeout: usize, WatchdogCode: u64, DataSize: usize, WatchdogData: *const u16) -> EfiStatus,
    pub ConnectController: usize,
    pub DisconnectController: usize,
    pub OpenProtocol: usize,
    pub CloseProtocol: usize,
    pub OpenProtocolInformation: usize,
    pub ProtocolsPerHandle: usize,
    pub LocateHandleBuffer: usize,
    pub LocateProtocol: unsafe extern "efiapi" fn(
        Protocol: *const EfiGuid,
        Registration: *const (),
        Interface: *mut *mut (),
    ) -> EfiStatus,
}
