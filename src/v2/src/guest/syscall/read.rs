// SPDX-License-Identifier: Apache-2.0

use super::Argv;
use crate::guest::alloc::{Allocator, Collector, Output, Syscall};
use crate::Result;

use libc::{c_int, c_long, size_t};

pub struct Read<'a> {
    pub fd: c_int,
    pub buf: &'a mut [u8],
}

unsafe impl<'a> Syscall<'a> for Read<'a> {
    const NUM: c_long = libc::SYS_read;

    type Argv = Argv<3>;
    type Ret = super::Result<size_t>;

    type Staged = Output<'a, [u8], &'a mut [u8]>;
    type Committed = Self::Staged;
    type Collected = Option<Result<size_t>>;

    fn stage(self, alloc: &mut impl Allocator) -> Result<(Self::Argv, Self::Staged)> {
        let (buf, _) = Output::stage_slice_max(alloc, self.buf)?;
        Ok((Argv([self.fd as _, buf.offset(), buf.len()]), buf))
    }

    fn collect(
        committed: Self::Committed,
        ret: Self::Ret,
        col: &impl Collector,
    ) -> Self::Collected {
        match ret.into() {
            Ok(ret) if ret > committed.len() => None,
            res @ Ok(ret) => {
                unsafe { committed.collect_range(col, 0..ret) };
                Some(res)
            }
            err => Some(err),
        }
    }
}
