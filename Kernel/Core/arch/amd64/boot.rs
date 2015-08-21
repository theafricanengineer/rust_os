// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// arch/amd64/boot.rs
//! Boot information.
//!
//! Parsing and exposure of the bootloader-provided data
#[allow(unused_imports)]
use prelude::*;
use super::memory::addresses::{IDENT_START, IDENT_END};
use metadevs::video::bootvideo::{VideoMode,VideoFormat};

#[repr(C)]
#[allow(unused)]
struct MultibootInfo
{
	flags: u32,
	// flags[0]
	lomem: u32, himem: u32,
	// flags[1]
	bootdev: u32,
	// flags[2]
	cmdline: u32,
	// flags[3]
	module_count: u32, module_first: u32,
	// flags[4] or flags[5]
	syminfo: [u32; 4],
	// flags[6]
	memmap_len: u32,
	memmap_ptr: u32,
	// flags[7]
	drives_length: u32,
	drives_addr: u32,
	// flags[8]
	configtable_ptr: u32,	// result of BIOS 'GET CONFIGURATION'
	// flags[9]
	boot_loader_name: u32,	// C string, booting loader
	// flags[10]
	apm_table_ptr: u32,
	// flags[11]
	vbe_control_info: u32,
	vbe_mode_info: u32,
	vbe_mode: u32,
	vbe_interface_seg: u32,
	vbe_interface_off: u32,
	vbe_interface_len: u32,
}

#[repr(C)]
#[allow(unused)]
#[derive(Debug)]
struct VbeModeInfo
{
	attributes: u16,
	window_attrs: [u8; 2],
	granuality: u16,
	window_size: u16,
	window_segments: [u16; 2],
	win_pos_fcn_fptr: [u16; 2],	// Pointer to INT 10h, AX=4F05h
	
	pitch: u16,
	x_res: u16, y_res: u16,
	char_w: u8, char_h: u8,
	n_planes: u8,
	bpp: u8,
	n_banks: u8,
	memory_model: u8,
	bank_size: u8,
	n_pages: u8,
	_resvd: u8,	// reserved
	
	// VBE 1.2+
	red_mask: u8,	red_position: u8,
	green_mask: u8, green_position: u8,
	blue_mask: u8,  blue_position: u8,
	rsv_mask: u8,   rsv_position: u8,
	directcolor_attributes: u8,

	// VBE v2.0+
	physbase: u32,
	offscreen_ptr: u32,	// Start of offscreen memory
	offscreen_size_kb: u16,	// Size of offscreen memory
	
	// -- VBE v3.0
	lfb_pitch: u16,
	image_count_banked: u8,
	image_count_lfb: u8,
}

struct MultibootParsed
{
	cmdline: &'static str,
	vidmode: Option<VideoMode>,
	memmap: &'static [::memory::MemoryMapEnt],
}

enum BootInfo
{
	Uninit,
	Invalid,
	Multiboot(MultibootParsed),
}


extern "C"
{
	static s_multiboot_signature : u32;
	static s_multiboot_pointer : &'static MultibootInfo;
}
static mut s_memmap_data: [::memory::MemoryMapEnt; 16] = [::memory::MAP_PAD; 16];
static mut s_bootinfo : BootInfo = BootInfo::Uninit;

fn get_bootinfo() -> &'static BootInfo
{
	unsafe
	{
		match s_bootinfo
		{
		BootInfo::Uninit => {
			s_bootinfo = match s_multiboot_signature
				{
				0x2BADB002 =>
					if let Some(mbi) = MultibootParsed::new(s_multiboot_pointer) {
						BootInfo::Multiboot(mbi)
					}
					else {
						BootInfo::Invalid
					},
				_ => BootInfo::Invalid,
				};
			},
		_ => {}
		}
		
		&s_bootinfo
	}
}

impl BootInfo
{
	pub fn cmdline(&self) -> &'static str
	{
		match *self
		{
		BootInfo::Uninit => "",
		BootInfo::Invalid => "",
		BootInfo::Multiboot(ref mb) => mb.cmdline
		}
	}
	
	pub fn vidmode(&self) -> Option<VideoMode>
	{
		match *self
		{
		BootInfo::Uninit => None,
		BootInfo::Invalid => None,
		BootInfo::Multiboot(ref mb) => mb.vidmode
		}
	}
	pub fn memmap(&self) -> &'static[::memory::MemoryMapEnt]
	{
		match *self
		{
		BootInfo::Uninit => &[],
		BootInfo::Invalid => &[],
		BootInfo::Multiboot(ref mb) => mb.memmap
		}
	}
}

fn valid_c_str_to_slice(ptr: *const i8) -> Option<&'static str>
{
	if let Some(s) = ::memory::c_string_as_byte_slice(ptr) {
		::core::str::from_utf8(s).ok()
	}
	else {
		None
	}
}

impl MultibootParsed
{
	pub fn new(info: &MultibootInfo) -> Option<MultibootParsed>
	{
		//if info.flags & !0xFFF != 0 {
		//	log_error!("Multiboot header malformed (reserved flag bits set {:#x})", info.flags);
		//	return None;
		//}
		let loader_name = if (info.flags & 1 << 9) != 0 {
				let loader_ptr = (info.boot_loader_name as usize + IDENT_START) as *const i8;
				log_debug!("loader_ptr = {:?}", loader_ptr);
				valid_c_str_to_slice(loader_ptr).unwrap_or("-INVALID-")
			}
			else {
				"-UNKNOWN-"
			};
		
		// Symbol information
		match (info.flags >> 4) & 3
		{
		0 => {},	// No symbol information
		1 => {
			// a.out symbol table
			let [tabsize, strsize, addr, _resvd] = info.syminfo;
			log_debug!("Symbols a.out - tabsize={}, strsize={}, addr={:#x}", tabsize, strsize, addr);
			},
		2 => {
			// Elf section header table
			if false
			{
				let [num, size, addr, shndx] = info.syminfo;
				log_debug!("Symbols ELF - num={}, size={}, addr={:#x}, shndx={}", num, size, addr, shndx);
				//let byte_ofs = addr as usize & (::PAGE_SIZE - 1);
				//let alloc = ::memory::virt::map_hw_ro(
				//	addr as ::memory::PAddr - byte_ofs as ::memory::PAddr,
				//	(byte_ofs as u32 + num*size) as usize, module_path!()
				//	).unwrap();
				//let buf = alloc.as_slice::<u8>(byte_ofs, (num*size) as usize);
				
				// SAFE: Memory is valid for this range (TODO: Use a checked version?)
				let buf: &'static [u8] = unsafe { ::core::slice::from_raw_parts( (addr as usize + IDENT_START) as *const u8, (num*size) as usize) };
				let sh = ::loading::elf::SectionHeader::from_ref(buf, size as usize, shndx as usize);
				sh.dump();
				sh.address_to_symbol(0);
			}
			},
		_ => {
			log_error!("Multiboot header malformed (both symbol table bits set)");
			return None;
			},
		}
		
		log_notice!("Loading multiboot from loader '{}' (flags = {:#x})", loader_name, info.flags);
		let mut ret = MultibootParsed {
				cmdline: MultibootParsed::_cmdline(info),
				vidmode: MultibootParsed::_vidmode(info),
				memmap: &[],
			};
		// SAFE: Should only be called before threading is initialised, so no race
		ret.memmap = unsafe { ret._memmap(info, &mut s_memmap_data) };
		Some( ret )
	}
	
	fn _cmdline(info: &MultibootInfo) -> &'static str
	{
		if (info.flags & 1 << 2) == 0 {
			return "";
		}
		
		let cmdline_paddr = info.cmdline as usize;
		if cmdline_paddr + IDENT_START >= IDENT_END {
			return "";
		}
		
		let charptr = (cmdline_paddr + IDENT_START) as *const i8;
		valid_c_str_to_slice(charptr).unwrap_or("-INVALID-")
	}
	
	fn _vidmode(info: &MultibootInfo) -> Option<VideoMode>
	{
		if (info.flags & 1 << 11) == 0 {
			log_notice!("get_video_mode - Video mode information not present");
			return None;
		}
		
		// SAFE: VBE info pointer should be valid for all of this call.
		let info = unsafe {
			let vbeinfo_vaddr = info.vbe_mode_info as usize + IDENT_START;
			if vbeinfo_vaddr + ::core::mem::size_of::<VbeModeInfo>() > IDENT_END {
				return None;
			}
			&*(vbeinfo_vaddr as *const VbeModeInfo)
			};
		
		log_trace!("MultibootInfo::_vidmode: info = {:?}", info);
		let pos_tuple = (info.red_position,info.green_position,info.blue_position);
		let size_tuple = (info.red_mask, info.green_mask, info.blue_mask);
		let fmt = match info.bpp
			{
			32 => match (pos_tuple, size_tuple)
				{
				((16,8,0), (8,8,8)) => VideoFormat::X8R8G8B8,	// 8:8:8:8 32BPP
				_ => todo!("MultibootInfo::_vidmode 32 pos={:?},size={:?}", pos_tuple, size_tuple),
				},
			24 => todo!("MultibootInfo::_vidmode: 24bpp"),
			16 => match (pos_tuple, size_tuple)
				{
				((11,5,0), (5,6,5)) => VideoFormat::R5G6B5,	// 5:6:5 16BPP
				//((10,5,0), (5,5,5)) => VideoFormat::X1R5G5B5,	// 5:5:5 15BPP
				_ => todo!("MultibootInfo::_vidmode: pos={:?},size={:?}", pos_tuple, size_tuple),
				},
			_ => {
				return None;
				},
			};
		
		Some( VideoMode {
			width: info.x_res,
			height: info.y_res,
			fmt: fmt,
			pitch: info.pitch as usize,
			base: info.physbase as ::arch::memory::PAddr,
			})
	}
	fn _memmap<'a>(&self, info: &MultibootInfo, buf: &'a mut[::memory::MemoryMapEnt]) -> &'a [::memory::MemoryMapEnt]
	{
		let size = {
			let mut mapbuilder = ::memory::MemoryMapBuilder::new(buf);
			// 1. Get raw map
			if false && (info.flags & 1 << 6) != 0 {
				// Full memory map
				panic!("TODO: Full memory map");
			}
			else if (info.flags & 1 << 0) != 0 {
				// Dumb memory map
				log_debug!("info = {{..., .lomem={}, .himem={} }}", info.lomem, info.himem);
				// - Low memory (before VGA BIOS)
				assert!(info.lomem >= 625);
				assert!(info.lomem <= 640);
				let top_lowmem = info.lomem as u64 * 1024;
				mapbuilder.append( 0x1000, top_lowmem - 0x1000, ::memory::MemoryState::Free, 0 );
				// - High memory (above 1MiB)
				mapbuilder.append( 0x100000, info.himem as u64 * 1024, ::memory::MemoryState::Free, 0 );
			}
			else {
				// No memory map
				panic!("TODO: Assumption memory map");
			}
			mapbuilder.sort();
			// TODO: Fix if not valid
			assert!( mapbuilder.validate() );
			
			// 2. Clobber out kernel, modules, and strings
			mapbuilder.set_range( 0x100000, &::arch::v_kernel_end as *const _ as u64 - IDENT_START as u64 - 0x10000,
				::memory::MemoryState::Used, 0 ).ok().unwrap();
			mapbuilder.set_range( self.cmdline.as_ptr() as u64 - IDENT_START as u64, self.cmdline.len() as u64,
				::memory::MemoryState::Used, 0 ).ok().unwrap();
			
			mapbuilder.size()
			};
		
		// 3. Return final result
		&buf[0 .. size]
	}
}

/// Retreive the multiboot "command line" string
pub fn get_boot_string() -> &'static str
{
	get_bootinfo().cmdline()
}

/// Obtain the boot video mode
pub fn get_video_mode() -> Option<VideoMode>
{
	get_bootinfo().vidmode()
}

/// Obtain the memory map
pub fn get_memory_map() -> &'static[::memory::MemoryMapEnt]
{
	get_bootinfo().memmap()
}

// vim: ft=rust

