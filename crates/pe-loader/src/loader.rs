//! PE binary loading and parsing

use anyhow::{Context, Result};
use goblin::pe::{self, PE, certificate_table, debug, header::CoffHeader, optional_header::OptionalHeader};
use log::{debug, error, info, log_enabled, Level};
use std::{collections::{HashMap, hash_map}, fs};

/// Load a PE binary from disk
pub fn load_binary(path: &str) -> Result<Vec<u8>> {
    env_logger::init();

    fs::read(path)
        .context("Failed to read PE binary")
        // validate that the length is at least 64 bytes
        .and_then(|data| {
            debug!("File size: {} bytes", data.len());
            if data.len() < 64 {
                Err(anyhow::anyhow!("PE binary too small"))
            // TODO: add upper limit on file sizes to prevent issues
            } else {
                Ok(data)
            }
        })
}

/// Parse PE headers
pub fn parse_pe(data: &[u8]) -> Result<PE> {
    debug!("Parsing PE headers");
    // check if the DOS MZ signature is present
    if &data[0..2] != b"MZ" {
        error!("Invalid DOS MZ signature");
        return Err(anyhow::anyhow!("Invalid DOS MZ signature"));
    }
    let pe_file_result = PE::parse(data)
        .context("Failed to parse PE binary")
        .expect("goblin PE parser failed to parse file");
    debug!("PE parsed successfully");

    let e_lfanew = pe_file_result.header.dos_header.pe_pointer as usize;
    debug!("e_lfanew (PE header offset): {:#X}", e_lfanew);
    if (e_lfanew + 4) > data.len() {
        error!("Invalid e_lfanew: points outside of file bounds");
        return Err(anyhow::anyhow!(
            "Invalid e_lfanew: points outside of file bounds"
        ));
    }
    debug!("PE offset within file bounds");
    debug!("Checking PE signature at offset {:#X}", e_lfanew);

    // check for PE signature
    if &data[e_lfanew..e_lfanew + 4] != b"PE\0\0" {
        error!("Invalid PE signature");
        return Err(anyhow::anyhow!("Invalid PE signature"));
    }
    debug!("PE signature valid");

    Ok(pe_file_result)
}

pub fn parse_coff_hdr(pe: &PE) -> Result<CoffHeader> {
    debug!("COFF Header:");
    debug!("  Machine: {:#X}", pe.header.coff_header.machine);

    if pe.header.coff_header.machine == goblin::pe::header::COFF_MACHINE_X86_64 {
        debug!("  Machine Type: x86_64");
    } else {
        debug!("  Machine Type: Unknown");
        return anyhow::bail!("Unsupported machine type");
    }

    debug!(
        "  Number of Sections: {}",
        pe.header.coff_header.number_of_sections
    );
    debug!(
        "  Time Date Stamp: {}",
        pe.header.coff_header.time_date_stamp
    );
    debug!(
        "  Pointer to Symbol Table: {:#X}",
        pe.header.coff_header.pointer_to_symbol_table
    );
    debug!(
        "  Number of Symbol Tables: {}",
        pe.header.coff_header.number_of_symbol_table
    );
    debug!(
        "  Size of Optional Header: {}",
        pe.header.coff_header.size_of_optional_header
    );
    if pe.header.coff_header.size_of_optional_header != 240 {
        return anyhow::bail!("Unexpected Optional Header size");
    }

    debug!(
        "  Characteristics: {:#X}",
        pe.header.coff_header.characteristics
    );
    parse_coff_header_characteristics(pe);

    Ok(pe.header.coff_header.clone())
}

pub fn parse_coff_header_characteristics(pe: &PE) {
    let mut characteristics_str = String::new();
    let characteristics = pe.header.coff_header.characteristics.clone();
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_RELOCS_STRIPPED != 0 {
        characteristics_str.push_str("IMAGE_FILE_RELOCS_STRIPPED | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_EXECUTABLE_IMAGE != 0 {
        characteristics_str.push_str("IMAGE_FILE_EXECUTABLE_IMAGE | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_LINE_NUMS_STRIPPED != 0 {
        characteristics_str.push_str("IMAGE_FILE_LINE_NUMS_STRIPPED | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_LOCAL_SYMS_STRIPPED != 0 {
        characteristics_str.push_str("IMAGE_FILE_LOCAL_SYMS_STRIPPED | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_AGGRESSIVE_WS_TRIM != 0 {
        characteristics_str.push_str("IMAGE_FILE_AGGRESSIVE_WS_TRIM | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_LARGE_ADDRESS_AWARE != 0 {
        characteristics_str.push_str("IMAGE_FILE_LARGE_ADDRESS_AWARE | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_32BIT_MACHINE != 0 {
        characteristics_str.push_str("IMAGE_FILE_32BIT_MACHINE | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_DEBUG_STRIPPED != 0 {
        characteristics_str.push_str("IMAGE_FILE_DEBUG_STRIPPED | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP != 0 {
        characteristics_str.push_str("IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_NET_RUN_FROM_SWAP != 0 {
        characteristics_str.push_str("IMAGE_FILE_NET_RUN_FROM_SWAP | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_SYSTEM != 0 {
        characteristics_str.push_str("IMAGE_FILE_SYSTEM | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_DLL != 0 {
        characteristics_str.push_str("IMAGE_FILE_DLL | ");
    }
    if characteristics & goblin::pe::characteristic::IMAGE_FILE_UP_SYSTEM_ONLY != 0 {
        characteristics_str.push_str("IMAGE_FILE_UP_SYSTEM_ONLY | ");
    }
    if characteristics_str.ends_with(" | ") {
        characteristics_str.truncate(characteristics_str.len() - 3); // Remove trailing " | "
    }
    debug!("  Characteristics Flags: {}", characteristics_str);
}

pub fn parse_opt_hdr(pe: &PE) -> Result<OptionalHeader> {

    let opt_hdr = pe.header.optional_header.expect("Optional header missing").clone();

    // check opt hdr magic eq 0x20B
    debug!("opt hdr magic: {:#X}", opt_hdr.standard_fields.magic);
    if opt_hdr.standard_fields.magic == goblin::pe::optional_header::IMAGE_NT_OPTIONAL_HDR64_MAGIC {
        debug!("PE Format: PE32+ (64-bit)");
    } else {
        return anyhow::bail!("Unsupported PE format");
    }

    // check linker version
    debug!("opt hdr linker version: {}.{}", 
        opt_hdr.standard_fields.major_linker_version,
        opt_hdr.standard_fields.minor_linker_version
    );

    // read code size
    debug!("opt hdr code size: {:#X}", 
        opt_hdr.standard_fields.size_of_code
    );

    // read data size
    debug!("opt hdr data size: {:#X}", 
        opt_hdr.standard_fields.size_of_initialized_data
    );

    // read bss size
    debug!("opt hdr bss size: {:#X}", 
        opt_hdr.standard_fields.size_of_uninitialized_data
    );

    // read entry point rva
    debug!("opt hdr entry point rva: {:#X}", 
        opt_hdr.standard_fields.address_of_entry_point
    );

    // read base of code RVA
    debug!("opt hdr base of code rva: {:#X}", 
        opt_hdr.standard_fields.base_of_code
    );

    // read image base address
    debug!("opt hdr image base addr: {:#X}", 
        opt_hdr.windows_fields.image_base
    );

    // read section alignment
    debug!("opt hdr section alignment: {:#X}", 
        opt_hdr.windows_fields.section_alignment
    );

    // read file alignment
    debug!("opt hdr file alignment: {:#X}", 
        opt_hdr.windows_fields.file_alignment
    );

    // read os and version
    debug!("opt hdr os version: {}.{}", 
        opt_hdr.windows_fields.major_operating_system_version,
        opt_hdr.windows_fields.minor_operating_system_version
    );

    // read image size
    debug!("opt hdr image size: {:#X}", 
        opt_hdr.windows_fields.size_of_image
    );

    // read header size
    debug!("opt hdr header size: {:#X}", 
        opt_hdr.windows_fields.size_of_headers
    );

    // read checksum
    debug!("opt hdr checksum: {:#X}", 
        opt_hdr.windows_fields.check_sum
    );

    // read subsystem and verify IMAGE_SUBSYSTEM_WINDOWS_CUI
    debug!("opt hdr subsystem: {:#X}", 
        opt_hdr.windows_fields.subsystem
    );
    if opt_hdr.windows_fields.subsystem != goblin::pe::subsystem::IMAGE_SUBSYSTEM_WINDOWS_CUI {
        return anyhow::bail!("Unsupported subsystem");
    }

    // read dll characteristics
    let dll_characteristics = opt_hdr.windows_fields.dll_characteristics;
    debug!("opt hdr dll characteristics: {:#X}", dll_characteristics);
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_NX_COMPAT != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_NX_COMPAT");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_NO_ISOLATION != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_NO_ISOLATION");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_NO_SEH != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_NO_SEH");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_NO_BIND != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_NO_BIND");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_APPCONTAINER != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_APPCONTAINER");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_WDM_DRIVER != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_WDM_DRIVER");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_GUARD_CF != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_GUARD_CF");
    }
    if dll_characteristics & pe::dll_characteristic::IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE != 0 {
        debug!("opt hdr dll char: IMAGE_DLLCHARACTERISTICS_TERMINAL_SERVER_AWARE");
    }

    // read stack reserve size
    debug!("opt hdr stack reserve size: {:#X}", 
        opt_hdr.windows_fields.size_of_stack_reserve
    );

    // read stack commit size
    debug!("opt hdr stack commit size: {:#X}", 
        opt_hdr.windows_fields.size_of_stack_commit
    );

    // read heap reserve size
    debug!("opt hdr heap reserve size: {:#X}", 
        opt_hdr.windows_fields.size_of_heap_reserve
    );

    // read heap commit size
    debug!("opt hdr heap commit size: {:#X}", 
        opt_hdr.windows_fields.size_of_heap_commit
    );

    // read number of data directories
    debug!("opt hdr number of data directories: {:#X}", 
        opt_hdr.windows_fields.number_of_rva_and_sizes
    );
    if opt_hdr.windows_fields.number_of_rva_and_sizes != 16 {
        return anyhow::bail!("Unexpected number of data directories");
    }
    Ok(opt_hdr)
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum DataDirectoryType {
    ExportTable,
    ImportTable,
    ResourceTable,
    ExceptionTable,
    CertificateTable,
    BaseRelocationTable,
    Debug,
    Architecture,
    GlobalPtr,
    TlsTable,
    LoadConfigTable,
    BoundImportTable,
    ImportAddressTable,
    DelayImportDescriptor,
    ClrRuntimeHeader,
    Reserved,
}

#[derive(Debug,Clone, PartialEq, Eq, Hash)]
pub struct DataDirEntry {
    pub present: bool,
    pub rva: u32,
    pub size: u32,
}

impl DataDirEntry {
    pub fn new(present: bool, rva: u32, size: u32) -> Self {
        DataDirEntry { present, rva, size }
    }

    pub fn new_empty() -> Self {
        DataDirEntry {
            present: false,
            rva: 0,
            size: 0,
        }
    }
}

pub fn parse_data_directories(pe: &PE) -> Result<HashMap<DataDirectoryType, DataDirEntry>> {
    let opt_hdr = pe.header.optional_header.as_ref().expect("Optional header missing");
    let mut out = HashMap::new();
    let data_dirs = &opt_hdr.data_directories;
    let export_tbl_dd_info = data_dirs.get_export_table();
    match export_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ExportTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ExportTable, DataDirEntry::new_empty()),
    };

    let import_tbl_dd_info = data_dirs.get_import_table();
    match import_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ImportTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ImportTable, DataDirEntry::new_empty()),
    };


    let rsrc_tbl_dd_info = data_dirs.get_resource_table();
    match rsrc_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ResourceTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ResourceTable, DataDirEntry::new_empty()),
    };

    let except_tbl_dd_info = data_dirs.get_exception_table();
    match except_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ExceptionTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ExceptionTable, DataDirEntry::new_empty()),
    };

    let certificate_table_dd_info = data_dirs.get_certificate_table();
    match certificate_table_dd_info {
        Some(dd) => out.insert(DataDirectoryType::CertificateTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::CertificateTable, DataDirEntry::new_empty()),
    };

    let base_reloc_tbl_dd_info = data_dirs.get_base_relocation_table();
    match base_reloc_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::BaseRelocationTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::BaseRelocationTable, DataDirEntry::new_empty()),
    };

    let debug_tbl_dd_info = data_dirs.get_debug_table();
    match debug_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::Debug, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::Debug, DataDirEntry::new_empty()),
    };

    let arch_tbl_dd_info = data_dirs.get_architecture();
    match arch_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::Architecture, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::Architecture, DataDirEntry::new_empty()),
    };

    let global_ptr_tbl_dd_info = data_dirs.get_global_ptr();
    match global_ptr_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::GlobalPtr, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::GlobalPtr, DataDirEntry::new_empty()),
    };

    let tls_tbl_dd_info = data_dirs.get_tls_table();
    match tls_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::TlsTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::TlsTable, DataDirEntry::new_empty()),
    };

    let load_config_tbl_dd_info = data_dirs.get_load_config_table();
    match load_config_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::LoadConfigTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::LoadConfigTable, DataDirEntry::new_empty()),
    };

    let bound_import_tbl_dd_info = data_dirs.get_bound_import_table();
    match bound_import_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::BoundImportTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::BoundImportTable, DataDirEntry::new_empty()),
    };

    let import_addr_tbl_dd_info = data_dirs.get_import_address_table();
    match import_addr_tbl_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ImportAddressTable, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ImportAddressTable, DataDirEntry::new_empty()),
    };

    let delay_import_dd_info = data_dirs.get_delay_import_descriptor();
    match delay_import_dd_info {
        Some(dd) => out.insert(DataDirectoryType::DelayImportDescriptor, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::DelayImportDescriptor, DataDirEntry::new_empty()),
    };

    let clr_runtime_hdr_dd_info = data_dirs.get_clr_runtime_header();
    match clr_runtime_hdr_dd_info {
        Some(dd) => out.insert(DataDirectoryType::ClrRuntimeHeader, DataDirEntry::new(true, dd.virtual_address, dd.size)),
        None => out.insert(DataDirectoryType::ClrRuntimeHeader, DataDirEntry::new_empty()),
    };

    if out[&DataDirectoryType::ImportTable].present {
        debug!("Import Table present at RVA {:#X} with size {:#X}", 
            out[&DataDirectoryType::ImportTable].rva,
            out[&DataDirectoryType::ImportTable].size
        );
    } else {
        error!("Import Table not present");
        return Err(anyhow::anyhow!("Import Table is required but not present"));
    }

    if out[&DataDirectoryType::BaseRelocationTable].present {
        debug!("Base Relocation Table present at RVA {:#X} with size {:#X}", 
            out[&DataDirectoryType::BaseRelocationTable].rva,
            out[&DataDirectoryType::BaseRelocationTable].size
        );
    } else {
        error!("Base Relocation Table not present");
        return Err(anyhow::anyhow!("Base Relocation Table is required but not present"));
    }   

    if out[&DataDirectoryType::ImportAddressTable].present {
        debug!("Import Address Table present at RVA {:#X} with size {:#X}", 
            out[&DataDirectoryType::ImportAddressTable].rva,
            out[&DataDirectoryType::ImportAddressTable].size
        );
    } else {
        error!("Import Address Table not present");
        return Err(anyhow::anyhow!("Import Address Table is required but not present"));
    }   

    
    Ok(out)
}