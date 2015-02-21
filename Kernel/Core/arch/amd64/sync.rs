// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// arch/amd64/sync.rs
// - Lightweight spinlock
use core::marker::{Send,Sync};

/// Lightweight protecting spinlock
pub struct Spinlock<T: Send>
{
	pub lock: ::core::atomic::AtomicBool,
	pub value: ::core::cell::UnsafeCell<T>,
}
unsafe impl<T: Send> Sync for Spinlock<T> {}

/// Handle to the held spinlock
pub struct HeldSpinlock<'lock,T:'lock+Send>
{
	lock: &'lock Spinlock<T>,
}

/// A handle for frozen interrupts
pub struct HeldInterrupts;

impl<T: Send> Spinlock<T>
{
	pub fn lock(&self) -> HeldSpinlock<T>
	{
		while self.lock.compare_and_swap(false, true, ::core::atomic::Ordering::Acquire) == true
		{
		}
		::core::atomic::fence(::core::atomic::Ordering::Acquire);
		HeldSpinlock { lock: self }
	}
	
	fn release(&self)
	{
		//::arch::puts("Spinlock::release()\n");
		::core::atomic::fence(::core::atomic::Ordering::Release);
		self.lock.store(false, ::core::atomic::Ordering::Release);
	}
}

#[unsafe_destructor]
impl<'lock,T: Send> ::core::ops::Drop for HeldSpinlock<'lock, T>
{
	fn drop(&mut self)
	{
		self.lock.release();
	}
}

impl<'lock,T: Send> ::core::ops::Deref for HeldSpinlock<'lock, T>
{
	type Target = T;
	fn deref<'a>(&'a self) -> &'a T {
		unsafe { &*self.lock.value.get() }
	}
}
impl<'lock,T: Send> ::core::ops::DerefMut for HeldSpinlock<'lock, T>
{
	fn deref_mut<'a>(&'a mut self) -> &'a mut T {
		unsafe { &mut *self.lock.value.get() }
	}
}

/// Prevent interrupts from firing until return value is dropped
pub fn hold_interrupts() -> HeldInterrupts
{
	let if_set = unsafe {
		let mut flags: u64;
		asm!("pushf; pop $0; cli" : "=r" (flags) : : "memory" : "volatile");
		(flags & 0x200) != 0
		};
	
	assert!(if_set, "Interupts clear when hold_interrupts called");
	
	HeldInterrupts
}

impl ::core::ops::Drop for HeldInterrupts
{
	fn drop(&mut self)
	{
		unsafe {
			asm!("sti" : : : "memory" : "volatile");
		}
	}
}

// vim: ft=rust

