// SPDX-License-Identifier: Apache-2.0

//! Host-specific functionality.

use crate::{item, SlicePtr};

use core::mem::size_of;
use core::ptr::{slice_from_raw_parts_mut, NonNull};
use libc::c_long;

struct Syscall<const N: usize, const M: usize> {
    /// The syscall number for the request.
    ///
    /// See, for example, [`libc::SYS_exit`](libc::SYS_exit).
    number: c_long,

    /// The syscall argument vector.
    argv: [usize; N],

    /// Return values.
    ret: *mut [usize; M],
}

trait Execute {
    unsafe fn execute(&mut self);
}

#[cfg(feature = "asm")]
mod asm {
    use super::{Execute, Syscall};

    impl Execute for Syscall<0, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<1, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<2, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            in("rsi") self.argv[1],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<3, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            in("rsi") self.argv[1],
            in("rdx") self.argv[2],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<4, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            in("rsi") self.argv[1],
            in("rdx") self.argv[2],
            in("r10") self.argv[3],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<5, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            in("rsi") self.argv[1],
            in("rdx") self.argv[2],
            in("r10") self.argv[3],
            in("r8") self.argv[4],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }

    impl Execute for Syscall<6, 1> {
        #[inline]
        unsafe fn execute(&mut self) {
            asm!(
            "syscall",
            inlateout("rax") self.number as usize => (*self.ret)[0],
            in("rdi") self.argv[0],
            in("rsi") self.argv[1],
            in("rdx") self.argv[2],
            in("r10") self.argv[3],
            in("r8") self.argv[4],
            in("r9") self.argv[5],
            lateout("rcx") _, // clobbered
            lateout("r11") _, // clobbered
            )
        }
    }
}

#[inline]
unsafe fn read_first<T>(ptr: *mut T) -> (T, *mut T) {
    (ptr.read(), ptr.add(1))
}

#[inline]
unsafe fn read_array<T, const N: usize>(ptr: *mut T) -> ([T; N], *mut T) {
    (ptr.cast::<[T; N]>().read(), ptr.add(N))
}

fn execute_item(ptr: *mut [u8]) -> Option<*mut [u8]> {
    let capacity = SlicePtr::len(ptr).checked_sub(2 * size_of::<usize>())?;

    let ptr = ptr as *mut usize;
    let (size, ptr) = unsafe { read_first(ptr) };
    let capacity = capacity.checked_sub(size)?;

    let (kind, ptr) = unsafe { read_first(ptr) };
    match kind {
        kind if kind == item::Kind::End as _ => return None,

        kind if kind == item::Kind::Syscall as _ => {
            let size = size.checked_sub(9 * size_of::<usize>())?;
            let (num, ptr) = unsafe { read_first(ptr) };
            match num {
                _ => (),
            }
        }
        _ => (),
    };
    Some(slice_from_raw_parts_mut(
        unsafe { ptr.cast::<u8>().add(size) },
        capacity,
    ))
}

/// Executes the passed `block`.
#[inline]
pub fn execute<const N: usize>(block: NonNull<[usize; N]>) {
    let ptr = slice_from_raw_parts_mut(block.as_ptr() as _, N * size_of::<usize>());
    execute_item(ptr);
}
