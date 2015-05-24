// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Core/main.rs
// - Kernel main
#![crate_name="kernel"]
#![crate_type="lib"]
#![feature(no_std)]
#![feature(asm)]	// Enables the asm! syntax extension
#![feature(box_syntax)]	// Enables 'box' syntax
#![feature(thread_local)]	// Allows use of thread_local
#![feature(lang_items)]	// Allow definition of lang_items
#![feature(core)]	// silences warnings about write!
#![feature(optin_builtin_traits)]	// Negative impls
#![feature(unique)]	// Unique
#![feature(slice_patterns)]	// Slice (array) destructuring patterns, used by multiboot code
#![feature(step_by)]	// Range::step_by
#![feature(linkage)]	// allows using #[linkage="external"]
#![no_std]

#![feature(plugin)]
#![feature(custom_attribute)]
#![plugin(tag_safe)]
use prelude::*;

#[macro_use]
extern crate core;

pub use arch::memory::PAGE_SIZE;

#[doc(hidden)]
#[macro_use] pub mod logmacros;
#[doc(hidden)]
#[macro_use] pub mod macros;
#[doc(hidden)]
#[macro_use] #[cfg(arch__amd64)] #[path="arch/amd64/mod-macros.rs"] pub mod arch_macros;

// Evil Hack: For some reason, write! (and friends) will expand pointing to std instead of core
#[doc(hidden)]
mod std {
	pub use core::option;
	pub use core::{default,fmt,cmp};
	pub use core::marker;	// needed for derive(Copy)
	pub use core::iter;	// needed for 'for'
}

/// Kernel's version of 'std::prelude'
pub mod prelude;

/// Library datatypes (Vec, Queue, ...)
#[macro_use]
pub mod lib;	// Clone of libstd

/// Heavy synchronisation primitives (Mutex, Semaphore, RWLock, ...)
#[macro_use]
pub mod sync;

/// Asynchrnous wait support
pub mod async;

/// Logging framework
pub mod logging;
/// Memory management (physical, virtual, heap)
pub mod memory;
/// Thread management
#[macro_use]
pub mod threads;
/// Timekeeping (timers and wall time)
pub mod time;

// Module/Executable loading (and symbol lookup)
pub mod loading;
/// Module management (loading and initialisation of kernel modules)
pub mod modules;

/// Meta devices (the Hardware Abstraction Layer)
pub mod metadevs;
/// Device to driver mapping manager
///
/// Starts driver instances for the devices it sees
pub mod device_manager;

/// User output, via a kernel-provided compositing "WM"
pub mod gui;

// Public for driver modules
pub mod vfs;

mod config;

/// Stack unwinding (panic) handling
pub mod unwind;

pub mod irqs;

/// Built-in device drivers
mod hw;

/// Achitecture-specific code - AMD64 (aka x86-64)
#[macro_use]
#[cfg(arch__amd64)] #[path="arch/amd64/mod.rs"] pub mod arch;	// Needs to be pub for exports to be avaliable

/// Kernel entrypoint
#[no_mangle]
pub extern "C" fn kmain()
{
	log_notice!("Tifflin Kernel v{} build {} starting", env!("TK_VERSION"), env!("TK_BUILD"));
	log_notice!("> Git state : {}", env!("TK_GITSPEC"));
	log_notice!("> Built with {}", env!("RUST_VERSION"));
	
	// Initialise core services before attempting modules
	::memory::phys::init();
	::memory::virt::init();
	::memory::heap::init();
	::threads::init();
	
	log_log!("Command line = '{}'", ::arch::boot::get_boot_string());
	// TODO: Parse command line into config
	
	// Dump active video mode
	let vidmode = ::arch::boot::get_video_mode();
	match vidmode {
	Some(m) => {
		log_debug!("Video mode : {}x{} @ {:#x}", m.width, m.height, m.base);
		::metadevs::video::set_boot_mode(m);
		},
	None => log_debug!("No video mode present")
	}
	
	// Modules (dependency tree included)
	// - Requests that the GUI be started as soon as possible
	::modules::init(&["GUI"]);
	
	// Yield to allow init threads to run
	::threads::yield_time();
	
	// Run system init
	sysinit();
	
	// Thread 0 idle loop
	log_info!("Entering idle");
	loop
	{
		log_trace!("TID0 napping");
		::threads::yield_time();
	}
}

// Initialise the system once drivers are up
fn sysinit()
{
	use metadevs::storage::VolumeHandle;
	use vfs::{mount,handle};
	use vfs::Path;
	
	// 1. Mount /system to the specified volume
	let sysdisk = ::config::get_string(::config::Value::SysDisk);
	match VolumeHandle::open_named(sysdisk)
	{
	Err(e) => {
		log_error!("Unable to open /system volume {}: {}", sysdisk, e);
		return ;
		},
	Ok(vh) => match mount::mount("/system".as_ref(), vh, "", &[])
		{
		Ok(_) => {},
		Err(e) => {
			log_error!("Unable to mount /system from {}: {:?}", sysdisk, e);
			return ;
			},
		},
	}
	
	// 2. Symbolic link /sysroot to the specified folder
	let sysroot = ::config::get_string(::config::Value::SysRoot);
	handle::Dir::open(Path::new("/")).unwrap().symlink("sysroot", Path::new(sysroot));

	// *. Testing: open a file known to exist on the testing disk	
	{
		match handle::File::open( Path::new("/system/1.TXT"), handle::FileOpenMode::SharedRO )
		{
		Err(e) => log_warning!("VFS test file can't be opened: {:?}", e),
		Ok(h) => {
			log_debug!("VFS open test = {:?}", h);
			let mut buf = [0; 16];
			let sz = h.read(0, &mut buf).unwrap();
			log_debug!("- Contents: {:?}", ::lib::RawString(&buf[..sz]));
			},
		}
		
		// - Iterate root dir
		match handle::Dir::open( Path::new("/") )
		{
		Err(e) => log_warning!("VFS root cannot be opened"),
		Ok(h) =>
			for name in h.iter() {
				log_log!("{}: {:?}", 0, name);
			},
		}
		// - Iterate /system
		match handle::Dir::open( Path::new("/system") )
		{
		Err(e) => log_warning!("System volume cannot be opened"),
		Ok(h) =>
			for name in h.iter() {
				log_log!("{:?}", name);
			},
		}
	}
	
}

// vim: ft=rust

