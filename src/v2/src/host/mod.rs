// SPDX-License-Identifier: Apache-2.0

//! Host-specific functionality.

use crate::{item, SlicePtr};
use core::marker::PhantomData;

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

/// Executes an item located at `ptr` and returns aligned pointer to next executable item on
/// success.
///
/// # Safety
///
/// `ptr` must be aligned to `align_of::<usize>()`.
///
fn execute_item(_item: BlockItem) {
    todo!()
}

/// Executes the passed `block`.
#[inline]
pub fn execute<const N: usize>(block: &mut [usize; N]) {
    for item in BlockIter::new(NonNull::from(block)) {
        execute_item(item)
    }
}

#[derive(Debug)]
struct BlockIter<'a, const N: usize> {
    capacity: usize,
    ptr: *mut usize,
    inner: NonNull<[usize; N]>,
    phantom: PhantomData<&'a ()>,
}

#[derive(Debug)]
struct BlockItem<'a> {
    pub kind: crate::item::Kind,
    pub ptr: *mut [u8],
    phantom: PhantomData<&'a ()>,
}

impl<const N: usize> BlockIter<'_, N> {
    pub fn new(block: NonNull<[usize; N]>) -> Self {
        Self {
            capacity: N * size_of::<usize>(),
            ptr: block.as_ptr() as _,
            inner: block,
            phantom: Default::default(),
        }
    }
}

impl<'a, const N: usize> Iterator for BlockIter<'a, N> {
    type Item = BlockItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let header: item::Header = unsafe { self.ptr.cast::<item::Header>().read() };

        if header.kind == item::Kind::End {
            assert_eq!(header.size, 0);
            return None;
        }

        if header.size % size_of::<usize>() != 0 {
            return None;
        }

        let skip = size_of::<item::Header>() + header.size;

        self.capacity = self.capacity.checked_sub(skip)?;

        let usize_len = size_of::<item::Header>() / size_of::<usize>();
        debug_assert_eq!(size_of::<item::Header>() % size_of::<usize>(), 0);
        self.ptr = unsafe { self.ptr.add(usize_len) };

        let ptr = self.ptr;

        let usize_len = header.size / size_of::<usize>();
        self.ptr = unsafe { self.ptr.add(usize_len) };

        dbg!(header.size);

        Some(BlockItem {
            kind: header.kind,
            ptr: slice_from_raw_parts_mut(ptr as *mut u8, header.size),
            phantom: Default::default(),
        })
    }
}

#[test]
fn test_iter() {
    let mut block: [usize; 20] = [32, 1, 0, 0, 0, 0, 24, 1, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];

    let mut iter = BlockIter::new(NonNull::from(&mut block));

    let next = iter.next().unwrap();
    assert!(matches!(next.kind, item::Kind::Syscall));
    assert_eq!(SlicePtr::len(next.ptr), 32);

    let next = iter.next().unwrap();
    assert!(matches!(next.kind, item::Kind::Syscall));
    assert_eq!(SlicePtr::len(next.ptr), 24);

    assert!(iter.next().is_none());
}
