//
//
//
//use prelude::*;

use self::_Unwind_Reason_Code::*;

#[allow(non_camel_case_types)]
#[repr(C)]
enum _Unwind_Reason_Code
{
	_URC_NO_REASON = 0,
	_URC_FOREIGN_EXCEPTION_CAUGHT = 1,
	_URC_FATAL_PHASE2_ERROR = 2,
	_URC_FATAL_PHASE1_ERROR = 3,
	_URC_NORMAL_STOP = 4,
	_URC_END_OF_STACK = 5,
	_URC_HANDLER_FOUND = 6,
	_URC_INSTALL_CONTEXT = 7,
	_URC_CONTINUE_UNWIND = 8,
}

#[allow(non_camel_case_types)]
struct _Unwind_Context;

#[allow(non_camel_case_types)]
type _Unwind_Action = u32;
static _UA_SEARCH_PHASE: _Unwind_Action = 1;

#[allow(non_camel_case_types)]
#[repr(C)]
struct _Unwind_Exception
{
	exception_class: u64,
	exception_cleanup: fn(_Unwind_Reason_Code,*const _Unwind_Exception),
	private: [u64; 2],
}

/*
#[repr(C)]
struct Exception
{
	header: _Unwind_Exception,
	cause: (),
}

extern "C" {
	fn _Unwind_RaiseException(ex: *const _Unwind_Exception) -> !;
}

static EXCEPTION_CLASS : u64 = 0x544B3120_52757374;	// TK1 Rust (big endian)
// */

// Evil fail when doing unwind
#[no_mangle] 
#[lang = "panic_fmt"]
pub extern "C" fn rust_begin_unwind(msg: ::core::fmt::Arguments, file: &'static str, line: usize) -> !
{
	static NESTED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::ATOMIC_BOOL_INIT;
	::arch::puts("\nERROR: rust_begin_unwind: ");
	::arch::puts(file);
	::arch::puts(":");
	::arch::puth(line as u64);
	::arch::puts("\n");
	if NESTED.swap(true, ::core::sync::atomic::Ordering::SeqCst) {
		::arch::puts("NESTED!\n");
		loop {}
	}
	::arch::print_backtrace();
	log_panic!("{}:{}: Panicked \"{:?}\"", file, line, msg);
	::metadevs::video::set_panic(file, line, msg);
	loop{}
}
#[lang="eh_personality"]
fn rust_eh_personality(
	version: isize, _actions: _Unwind_Action, _exception_class: u64,
	_exception_object: &_Unwind_Exception, _context: &_Unwind_Context
	) -> _Unwind_Reason_Code
{
	log_debug!("rust_eh_personality(version={},_actions={},_exception_class={:#x})",
		version, _actions, _exception_class);
	if version != 1 {
		log_error!("version({}) != 1", version);
		return _URC_FATAL_PHASE1_ERROR;
	}
	loop{}
}

#[no_mangle] pub extern "C" fn abort() -> !
{
	::arch::puts("\nABORT ABORT ABORT\n");
	::arch::print_backtrace();
	loop {}
}

// vim: ft=rust
