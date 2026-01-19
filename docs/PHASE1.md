# Phase 1: Userspace Prototype - Detailed Task Checklist

**Goal**: Build a complete PE loader that runs in Linux userspace, allowing us to test and debug the loader logic before porting to bare-metal.

**Location**: `crates/pe-loader/`

**Success Criteria**: Load `target-zero.exe` and execute it successfully, producing the expected output.

---

## 1. Project Setup and Infrastructure

### 1.1 Development Environment
- [ ] **1.1.1** Verify Rust toolchain is installed and up to date
- [ ] **1.1.2** Install and configure logging framework (env_logger)
- [ ] **1.1.3** Set up integration test infrastructure
- [ ] **1.1.4** Create test fixtures directory for PE binaries

### 1.2 Error Handling
- [ ] **1.2.1** Define custom error types in `crates/pe-loader/src/error.rs`
  - [ ] ParseError - For PE parsing failures
  - [ ] LoadError - For memory loading failures
  - [ ] RelocError - For relocation failures
  - [ ] ImportError - For import resolution failures
  - [ ] ExecutionError - For execution failures
- [ ] **1.2.2** Implement Display and Error traits
- [ ] **1.2.3** Add context to all error types

---

## 2. Binary Loading (PE Parsing)

**File**: `crates/pe-loader/src/loader.rs`

### 2.1 File Reading
- [ ] **2.1.1** Implement `load_binary(path: &str) -> Result<Vec<u8>>`
- [ ] **2.1.2** Add file size validation (min: 64 bytes for DOS header)
- [ ] **2.1.3** Add file size limit check (e.g., max 100MB for safety)
- [ ] **2.1.4** Add detailed logging for file operations

### 2.2 DOS Header Parsing
- [ ] **2.2.1** Validate DOS signature (`MZ` / 0x5A4D)
- [ ] **2.2.2** Read `e_lfanew` offset (PE header location)
- [ ] **2.2.3** Validate `e_lfanew` is within file bounds
- [ ] **2.2.4** Log DOS header information

### 2.3 PE Signature Validation
- [ ] **2.3.1** Read 4-byte PE signature at `e_lfanew` offset
- [ ] **2.3.2** Validate signature is `PE\0\0` (0x00004550)
- [ ] **2.3.3** Log PE signature validation result

### 2.4 COFF Header Parsing
- [ ] **2.4.1** Parse COFF header structure
- [ ] **2.4.2** Validate machine type is `IMAGE_FILE_MACHINE_AMD64` (0x8664)
- [ ] **2.4.3** Read number of sections
- [ ] **2.4.4** Read size of optional header
- [ ] **2.4.5** Validate optional header size (should be 240 for PE32+)
- [ ] **2.4.6** Read and log characteristics flags
- [ ] **2.4.7** Log COFF header details

### 2.5 Optional Header Parsing (PE32+)
- [ ] **2.5.1** Validate magic number is 0x20B (PE32+)
- [ ] **2.5.2** Read linker version
- [ ] **2.5.3** Read code size, data size, BSS size
- [ ] **2.5.4** Read entry point RVA (Relative Virtual Address)
- [ ] **2.5.5** Read base of code RVA
- [ ] **2.5.6** Read image base address (preferred load address)
- [ ] **2.5.7** Read section alignment
- [ ] **2.5.8** Read file alignment
- [ ] **2.5.9** Read OS and subsystem version
- [ ] **2.5.10** Read image size (total size when loaded)
- [ ] **2.5.11** Read headers size
- [ ] **2.5.12** Read checksum (can ignore for now)
- [ ] **2.5.13** Read subsystem (should be IMAGE_SUBSYSTEM_WINDOWS_CUI for console)
- [ ] **2.5.14** Read DLL characteristics
- [ ] **2.5.15** Read stack reserve/commit sizes
- [ ] **2.5.16** Read heap reserve/commit sizes
- [ ] **2.5.17** Read number of data directories (should be 16)
- [ ] **2.5.18** Log all optional header fields

### 2.6 Data Directories Parsing
- [ ] **2.6.1** Parse all 16 data directory entries:
  - [ ] Export Table (index 0)
  - [ ] Import Table (index 1) **[CRITICAL]**
  - [ ] Resource Table (index 2)
  - [ ] Exception Table (index 3)
  - [ ] Certificate Table (index 4)
  - [ ] Base Relocation Table (index 5) **[CRITICAL]**
  - [ ] Debug (index 6)
  - [ ] Architecture (index 7)
  - [ ] Global Ptr (index 8)
  - [ ] TLS Table (index 9)
  - [ ] Load Config Table (index 10)
  - [ ] Bound Import (index 11)
  - [ ] IAT (index 12) **[CRITICAL]**
  - [ ] Delay Import Descriptor (index 13)
  - [ ] CLR Runtime Header (index 14)
  - [ ] Reserved (index 15)
- [ ] **2.6.2** Store RVA and size for each directory
- [ ] **2.6.3** Validate critical directories exist (Import, Reloc, IAT)
- [ ] **2.6.4** Log data directory information

### 2.7 Section Headers Parsing
- [ ] **2.7.1** Read all section headers (number from COFF header)
- [ ] **2.7.2** For each section, parse:
  - [ ] Name (8 bytes, null-padded)
  - [ ] Virtual size (size in memory)
  - [ ] Virtual address (RVA where section loads)
  - [ ] Size of raw data (size in file)
  - [ ] Pointer to raw data (file offset)
  - [ ] Pointer to relocations (usually 0 for executables)
  - [ ] Pointer to line numbers (usually 0)
  - [ ] Number of relocations
  - [ ] Number of line numbers
  - [ ] Characteristics flags
- [ ] **2.7.3** Validate section alignments
- [ ] **2.7.4** Validate section file offsets are within file bounds
- [ ] **2.7.5** Validate section virtual addresses don't overlap
- [ ] **2.7.6** Identify critical sections (.text, .data, .rdata, .reloc, .idata)
- [ ] **2.7.7** Log section table information

### 2.8 PE Structure Wrapper
- [ ] **2.8.1** Create `LoadedPE` struct to hold parsed data
- [ ] **2.8.2** Implement helper methods:
  - [ ] `get_section_by_rva(rva: u32) -> Option<&Section>`
  - [ ] `get_section_by_name(name: &str) -> Option<&Section>`
  - [ ] `rva_to_file_offset(rva: u32) -> Option<usize>`
  - [ ] `rva_to_va(rva: u32, base: usize) -> usize`
- [ ] **2.8.3** Add comprehensive debug output

---

## 3. Memory Mapping

**File**: `crates/pe-loader/src/memory.rs`

### 3.1 Memory Allocation Strategy
- [ ] **3.1.1** Calculate total image size from optional header
- [ ] **3.1.2** Determine if we can allocate at preferred base address
- [ ] **3.1.3** If not, find alternative address (will require relocations)
- [ ] **3.1.4** Log allocation strategy and chosen base address

### 3.2 Base Memory Allocation
- [ ] **3.2.1** Implement `allocate_image_base(size: usize, preferred_base: u64) -> Result<*mut u8>`
- [ ] **3.2.2** Use `mmap` to allocate at specific address:
  ```rust
  mmap(
      addr: preferred_base,
      length: size,
      prot: PROT_READ | PROT_WRITE,
      flags: MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED_NOREPLACE,
      fd: -1,
      offset: 0
  )
  ```
- [ ] **3.2.3** Handle MAP_FIXED_NOREPLACE failure (address not available)
- [ ] **3.2.4** Fall back to letting kernel choose address if preferred fails
- [ ] **3.2.5** Validate allocation succeeded
- [ ] **3.2.6** Zero the allocated memory
- [ ] **3.2.7** Log allocation details

### 3.3 Header Mapping
- [ ] **3.3.1** Copy DOS header to base address
- [ ] **3.3.2** Copy PE headers (up to first section)
- [ ] **3.3.3** Validate headers size from optional header
- [ ] **3.3.4** Log header mapping

### 3.4 Section Mapping
- [ ] **3.4.1** For each section in section table:
  - [ ] **3.4.1.1** Calculate destination VA: `base + section.virtual_address`
  - [ ] **3.4.1.2** Get source data from file at `section.pointer_to_raw_data`
  - [ ] **3.4.1.3** Copy `min(section.size_of_raw_data, section.virtual_size)` bytes
  - [ ] **3.4.1.4** If virtual_size > size_of_raw_data, zero remaining bytes (BSS)
  - [ ] **3.4.1.5** Validate copy succeeded
  - [ ] **3.4.1.6** Log section mapping (name, VA, size)

### 3.5 Memory Protection
- [ ] **3.5.1** Implement `set_section_protection(addr: *mut u8, size: usize, characteristics: u32) -> Result<()>`
- [ ] **3.5.2** Map PE characteristics to mprotect flags:
  - [ ] IMAGE_SCN_MEM_EXECUTE → PROT_EXEC
  - [ ] IMAGE_SCN_MEM_READ → PROT_READ
  - [ ] IMAGE_SCN_MEM_WRITE → PROT_WRITE
- [ ] **3.5.3** Apply protections to each section
- [ ] **3.5.4** Common combinations:
  - [ ] .text: PROT_READ | PROT_EXEC (RX)
  - [ ] .rdata: PROT_READ (R)
  - [ ] .data: PROT_READ | PROT_WRITE (RW)
- [ ] **3.5.5** Use `mprotect` system call
- [ ] **3.5.6** Handle mprotect errors
- [ ] **3.5.7** Log protection settings for each section

### 3.6 Memory Layout Validation
- [ ] **3.6.1** Implement validation function
- [ ] **3.6.2** Verify all sections are within image bounds
- [ ] **3.6.3** Verify no sections overlap
- [ ] **3.6.4** Verify alignment requirements are met
- [ ] **3.6.5** Log complete memory layout

---

## 4. Base Relocations

**File**: `crates/pe-loader/src/reloc.rs`

### 4.1 Relocation Decision
- [ ] **4.1.1** Calculate base delta: `actual_base - preferred_base`
- [ ] **4.1.2** If delta is 0, skip relocations (loaded at preferred address)
- [ ] **4.1.3** If delta is non-zero, apply relocations
- [ ] **4.1.4** Log relocation decision and delta

### 4.2 Relocation Directory Parsing
- [ ] **4.2.1** Get base relocation data directory from optional header
- [ ] **4.2.2** Convert RVA to VA in mapped memory
- [ ] **4.2.3** Validate relocation directory exists
- [ ] **4.2.4** Validate relocation directory is within .reloc section

### 4.3 Relocation Block Processing
- [ ] **4.3.1** Parse IMAGE_BASE_RELOCATION structure:
  ```rust
  struct ImageBaseRelocation {
      virtual_address: u32,  // RVA of block
      size_of_block: u32,    // Size including entries
  }
  ```
- [ ] **4.3.2** For each relocation block:
  - [ ] **4.3.2.1** Read block header (virtual_address, size_of_block)
  - [ ] **4.3.2.2** Calculate number of entries: `(size_of_block - 8) / 2`
  - [ ] **4.3.2.3** Read all 16-bit relocation entries
  - [ ] **4.3.2.4** Parse each entry:
    - [ ] Type (upper 4 bits)
    - [ ] Offset (lower 12 bits)
  - [ ] **4.3.2.5** Log block information

### 4.4 Relocation Application
- [ ] **4.4.1** For each relocation entry in block:
  - [ ] **4.4.1.1** Calculate target RVA: `block.virtual_address + entry.offset`
  - [ ] **4.4.1.2** Convert RVA to VA: `base + target_rva`
  - [ ] **4.4.1.3** Handle relocation type:
    - [ ] **IMAGE_REL_BASED_ABSOLUTE** (0) - Skip, used for padding
    - [ ] **IMAGE_REL_BASED_HIGH** (1) - Add high 16 bits of delta
    - [ ] **IMAGE_REL_BASED_LOW** (2) - Add low 16 bits of delta
    - [ ] **IMAGE_REL_BASED_HIGHLOW** (3) - Add all 32 bits of delta
    - [ ] **IMAGE_REL_BASED_DIR64** (10) - Add all 64 bits of delta **[MOST COMMON FOR x64]**
  - [ ] **4.4.1.4** Read current value at target address
  - [ ] **4.4.1.5** Add base delta to current value
  - [ ] **4.4.1.6** Write updated value back
  - [ ] **4.4.1.7** Validate write succeeded
- [ ] **4.4.2** Count and log total relocations applied

### 4.5 Relocation Validation
- [ ] **4.5.1** Verify all relocation targets are within valid sections
- [ ] **4.5.2** Verify no relocations in non-writable sections (would fail)
- [ ] **4.5.3** Log relocation summary

---

## 5. Import Resolution (IAT Patching)

**File**: `crates/pe-loader/src/imports.rs`

### 5.1 Symbol Resolver Setup
- [ ] **5.1.1** Expand `SymbolResolver` struct
- [ ] **5.1.2** Add support for multiple DLLs (HashMap<String, HashMap<String, usize>>)
- [ ] **5.1.3** Implement case-insensitive DLL name lookup
- [ ] **5.1.4** Add detailed logging

### 5.2 kernel32.dll Implementation
- [ ] **5.2.1** Register `GetStdHandle` implementation
  - [ ] Map STD_INPUT_HANDLE → fake handle 0x10
  - [ ] Map STD_OUTPUT_HANDLE → fake handle 0x11
  - [ ] Map STD_ERROR_HANDLE → fake handle 0x12
  - [ ] Test function standalone
- [ ] **5.2.2** Register `WriteFile` implementation
  - [ ] Validate handle (0x11 or 0x12 for stdout/stderr)
  - [ ] Extract buffer and length from parameters
  - [ ] Write to stdout using Rust's println! or write!
  - [ ] Update bytes_written pointer
  - [ ] Return success (1) or failure (0)
  - [ ] Test function standalone
- [ ] **5.2.3** Register `ExitProcess` implementation
  - [ ] Extract exit code from parameter
  - [ ] Log exit code
  - [ ] Call `std::process::exit(code)`
  - [ ] Test function standalone
- [ ] **5.2.4** Register `GetLastError` implementation
  - [ ] Return thread-local error code
  - [ ] Test function standalone
- [ ] **5.2.5** Register `SetLastError` implementation
  - [ ] Set thread-local error code
  - [ ] Test function standalone

### 5.3 Import Directory Parsing
- [ ] **5.3.1** Get import data directory from optional header
- [ ] **5.3.2** Convert RVA to VA in mapped memory
- [ ] **5.3.3** Parse import directory as array of IMAGE_IMPORT_DESCRIPTOR:
  ```rust
  struct ImageImportDescriptor {
      original_first_thunk: u32,  // RVA to ILT
      time_date_stamp: u32,
      forwarder_chain: u32,
      name: u32,                  // RVA to DLL name
      first_thunk: u32,           // RVA to IAT
  }
  ```
- [ ] **5.3.4** Iterate until null descriptor (all fields zero)
- [ ] **5.3.5** For each import descriptor:
  - [ ] **5.3.5.1** Read DLL name from RVA
  - [ ] **5.3.5.2** Validate DLL name is null-terminated
  - [ ] **5.3.5.3** Convert to lowercase for comparison
  - [ ] **5.3.5.4** Log DLL name

### 5.4 Import Name Resolution
- [ ] **5.4.1** For each import descriptor:
  - [ ] **5.4.1.1** Get ILT (Import Lookup Table) pointer from `original_first_thunk`
  - [ ] **5.4.1.2** Get IAT (Import Address Table) pointer from `first_thunk`
  - [ ] **5.4.1.3** If ILT is 0, use IAT as ILT (some linkers do this)
  - [ ] **5.4.1.4** Walk ILT/IAT as array of 64-bit entries (for PE32+)
  - [ ] **5.4.1.5** For each entry (until null entry):
    - [ ] **5.4.1.5.1** Check MSB (bit 63)
    - [ ] **5.4.1.5.2** If MSB set: Import by ordinal
      - [ ] Extract ordinal (lower 16 bits)
      - [ ] Log ordinal import
      - [ ] Resolve ordinal (not implemented for Phase 1)
      - [ ] Return error if ordinal imports are required
    - [ ] **5.4.1.5.3** If MSB clear: Import by name
      - [ ] Entry is RVA to IMAGE_IMPORT_BY_NAME structure
      - [ ] Read hint (16-bit ordinal hint)
      - [ ] Read name (null-terminated string)
      - [ ] Log import name
      - [ ] Look up name in SymbolResolver
      - [ ] Get function address
    - [ ] **5.4.1.5.4** Handle missing imports:
      - [ ] Option A: Return error (strict mode)
      - [ ] Option B: Provide stub function that logs and returns error
      - [ ] Make configurable
    - [ ] **5.4.1.5.5** Patch IAT entry with resolved address
    - [ ] **5.4.1.5.6** Log resolved import (name → address)

### 5.5 IAT Protection
- [ ] **5.5.1** After patching, optionally set IAT to read-only
- [ ] **5.5.2** Use mprotect to change .idata or IAT section to PROT_READ
- [ ] **5.5.3** Log protection change
- [ ] **5.5.4** Make this optional (some binaries write to IAT at runtime)

### 5.6 Import Validation
- [ ] **5.6.1** Verify all imports were resolved
- [ ] **5.6.2** Count total imports per DLL
- [ ] **5.6.3** Log summary statistics
- [ ] **5.6.4** Fail if critical imports are missing

---

## 6. Execution Setup

**File**: `crates/pe-loader/src/executor.rs`

### 6.1 Thread Environment Block (TEB) Setup
- [ ] **6.1.1** Allocate TEB structure (from common crate)
- [ ] **6.1.2** Initialize TEB fields:
  - [ ] `peb` pointer → point to PEB
  - [ ] `last_error` → 0
  - [ ] `thread_id` → 1 (fake TID)
  - [ ] `process_id` → 1 (fake PID)
- [ ] **6.1.3** Store TEB pointer for later

### 6.2 Process Environment Block (PEB) Setup
- [ ] **6.2.1** Allocate PEB structure
- [ ] **6.2.2** Initialize PEB fields:
  - [ ] `image_base_address` → actual base address of loaded PE
  - [ ] `process_heap` → fake heap handle (e.g., 1)
  - [ ] `process_parameters` → null for now (Phase 5)
- [ ] **6.2.3** Update TEB to point to PEB

### 6.3 Stack Allocation
- [ ] **6.3.1** Allocate stack memory (e.g., 1 MB)
- [ ] **6.3.2** Use mmap with PROT_READ | PROT_WRITE
- [ ] **6.3.3** Ensure 16-byte alignment (required by x64 ABI)
- [ ] **6.3.4** Calculate stack top (grows downward)
- [ ] **6.3.5** Optionally add guard page at bottom
- [ ] **6.3.6** Log stack allocation (base, top, size)

### 6.4 Register Preparation
- [ ] **6.4.1** Calculate entry point: `base + entry_point_rva`
- [ ] **6.4.2** Validate entry point is within .text section
- [ ] **6.4.3** Prepare initial register state:
  - [ ] RSP → stack top (16-byte aligned)
  - [ ] RBP → 0 (no frame yet)
  - [ ] RIP → entry point
  - [ ] RCX, RDX, R8, R9 → 0 (Windows x64 calling convention)
  - [ ] Other registers → 0
- [ ] **6.4.4** For now, we can't actually set registers in userspace

### 6.5 Entry Point Invocation
- [ ] **6.5.1** Cast entry point to function pointer:
  ```rust
  type EntryPoint = extern "C" fn() -> !;
  let entry: EntryPoint = unsafe { std::mem::transmute(entry_point_addr) };
  ```
- [ ] **6.5.2** Set up signal handlers for crashes (SIGSEGV, SIGILL, etc.)
- [ ] **6.5.3** Log execution start
- [ ] **6.5.4** Call entry point function
- [ ] **6.5.5** Handle return (shouldn't return, but log if it does)

---

## 7. Testing and Validation

**File**: `crates/pe-loader/tests/`

### 7.1 Unit Tests
- [ ] **7.1.1** Test DOS header validation (valid, invalid magic)
- [ ] **7.1.2** Test PE signature validation
- [ ] **7.1.3** Test COFF header parsing
- [ ] **7.1.4** Test optional header parsing
- [ ] **7.1.5** Test section header parsing
- [ ] **7.1.6** Test RVA to file offset conversion
- [ ] **7.1.7** Test relocation parsing
- [ ] **7.1.8** Test import descriptor parsing

### 7.2 Integration Tests
- [ ] **7.2.1** Test loading target-zero.exe
- [ ] **7.2.2** Verify all headers parse correctly
- [ ] **7.2.3** Verify sections map correctly
- [ ] **7.2.4** Verify relocations apply correctly (if needed)
- [ ] **7.2.5** Verify imports resolve correctly
- [ ] **7.2.6** Verify execution produces correct output

### 7.3 Target Zero Validation
- [ ] **7.3.1** Run pe-loader with target-zero.exe
- [ ] **7.3.2** Verify output: "Hello from Target Zero!"
- [ ] **7.3.3** Verify exit code: 0
- [ ] **7.3.4** Verify no crashes or errors

### 7.4 Error Handling Tests
- [ ] **7.4.1** Test with non-PE file (should error gracefully)
- [ ] **7.4.2** Test with corrupted PE file
- [ ] **7.4.3** Test with missing import (should error or provide stub)
- [ ] **7.4.4** Test with invalid section alignment
- [ ] **7.4.5** Test with invalid entry point

### 7.5 Performance Tests
- [ ] **7.5.1** Measure load time
- [ ] **7.5.2** Measure parse time
- [ ] **7.5.3** Measure relocation time
- [ ] **7.5.4** Log performance metrics

---

## 8. Documentation and Cleanup

### 8.1 Code Documentation
- [ ] **8.1.1** Add rustdoc comments to all public functions
- [ ] **8.1.2** Add module-level documentation
- [ ] **8.1.3** Add examples in documentation
- [ ] **8.1.4** Document PE format structures
- [ ] **8.1.5** Document Windows calling conventions used

### 8.2 User Documentation
- [ ] **8.2.1** Update README with Phase 1 completion
- [ ] **8.2.2** Add usage examples
- [ ] **8.2.3** Document command-line options
- [ ] **8.2.4** Add troubleshooting section

### 8.3 Code Quality
- [ ] **8.3.1** Run cargo fmt
- [ ] **8.3.2** Run cargo clippy and fix warnings
- [ ] **8.3.3** Run cargo test and ensure all tests pass
- [ ] **8.3.4** Check for TODO comments and address them
- [ ] **8.3.5** Remove debug print statements
- [ ] **8.3.6** Verify proper error handling everywhere

### 8.4 Final Validation
- [ ] **8.4.1** Build in release mode
- [ ] **8.4.2** Test with target-zero.exe
- [ ] **8.4.3** Verify clean output
- [ ] **8.4.4** Commit changes
- [ ] **8.4.5** Update ROADMAP.md to mark Phase 1 complete

---

## Success Criteria Checklist

Phase 1 is complete when:

- [ ] **S1** target-zero.exe loads without errors
- [ ] **S2** All PE headers parse correctly
- [ ] **S3** All sections map to correct virtual addresses
- [ ] **S4** Relocations apply correctly (if base address differs)
- [ ] **S5** All imports resolve to Rust implementations
- [ ] **S6** IAT is patched with correct function addresses
- [ ] **S7** Execution reaches entry point
- [ ] **S8** Output: "Hello from Target Zero!" appears on stdout
- [ ] **S9** Process exits with code 0
- [ ] **S10** No crashes, segfaults, or undefined behavior
- [ ] **S11** All unit tests pass
- [ ] **S12** All integration tests pass
- [ ] **S13** Code is well-documented
- [ ] **S14** Code passes clippy and fmt checks

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. Setup | 4 | Low |
| 2. PE Parsing | 45 | Medium-High |
| 3. Memory Mapping | 30 | Medium |
| 4. Relocations | 25 | Medium-High |
| 5. Imports | 35 | High |
| 6. Execution | 20 | Medium |
| 7. Testing | 20 | Medium |
| 8. Documentation | 15 | Low |
| **Total** | **~194 tasks** | **Mixed** |

---

## Dependencies Between Tasks

```
Setup (1) → PE Parsing (2) → Memory Mapping (3)
                            ↓
PE Parsing (2) → Relocations (4) → Execution (6)
                            ↓
PE Parsing (2) → Imports (5) → Execution (6)
                            ↓
All Above → Testing (7) → Documentation (8)
```

---

## Next Steps

After Phase 1 is complete:
1. Move to Phase 2: Port loader to bare-metal kernel
2. Implement memory management for kernel environment
3. Integrate with bootloader

See [PHASE2.md](PHASE2.md) for next phase details.
