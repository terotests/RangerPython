// SPDX-License-Identifier: MIT
//! A small-object allocator in front of the system one, as CPython has
//! pymalloc: the interpreter makes and drops many small blocks (strs, list
//! and tuple buffers, frames), and a free list per size class serves those
//! in a few instructions where malloc and free take many more.
//!
//! Blocks of 1 to 256 bytes (alignment at most 16) come in 16-byte size
//! classes, carved from 64 KiB chunks. A freed block goes on its class's
//! free list in the thread that frees it and is never given back to the
//! system; larger blocks go to the system allocator as they are.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr::null_mut;

const CLASSES: usize = 16;
const MAX_SMALL: usize = CLASSES * 16;
const CHUNK: usize = 64 * 1024;

struct Pools {
    free: [Cell<*mut u8>; CLASSES],
    bump: Cell<*mut u8>,
    end: Cell<*mut u8>,
}

thread_local! {
    static POOLS: Pools = const {
        Pools { free: [const { Cell::new(null_mut()) }; CLASSES], bump: Cell::new(null_mut()), end: Cell::new(null_mut()) }
    };
}

pub struct PoolAlloc;

#[inline(always)]
fn is_small(l: &Layout) -> bool {
    l.size() <= MAX_SMALL && l.align() <= 16
}

#[inline(always)]
fn class_of(size: usize) -> usize {
    (size.max(1) + 15) / 16 - 1
}

impl Pools {
    #[inline(always)]
    unsafe fn take(&self, c: usize) -> *mut u8 {
        let head = self.free[c].get();
        if !head.is_null() {
            self.free[c].set(*(head as *mut *mut u8));
            return head;
        }
        self.carve(c)
    }

    #[inline(never)]
    unsafe fn carve(&self, c: usize) -> *mut u8 {
        let size = (c + 1) * 16;
        let mut b = self.bump.get();
        if b.is_null() || (self.end.get() as usize) - (b as usize) < size {
            b = System.alloc(Layout::from_size_align_unchecked(CHUNK, 16));
            if b.is_null() {
                return b;
            }
            self.end.set(b.add(CHUNK));
        }
        self.bump.set(b.add(size));
        b
    }

    #[inline(always)]
    unsafe fn give(&self, p: *mut u8, c: usize) {
        *(p as *mut *mut u8) = self.free[c].get();
        self.free[c].set(p);
    }
}

unsafe impl GlobalAlloc for PoolAlloc {
    #[inline]
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if !is_small(&l) {
            return System.alloc(l);
        }
        let c = class_of(l.size());
        match POOLS.try_with(|p| p.take(c)) {
            Ok(p) => p,
            // a block from the system serves as well: it is never freed to it
            Err(_) => System.alloc(Layout::from_size_align_unchecked((c + 1) * 16, 16)),
        }
    }

    #[inline]
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        if !is_small(&l) {
            return System.dealloc(p, l);
        }
        // after the thread's pools are gone the block is let go of
        let _ = POOLS.try_with(|pools| pools.give(p, class_of(l.size())));
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        if !is_small(&l) {
            return System.alloc_zeroed(l);
        }
        let p = self.alloc(l);
        if !p.is_null() {
            std::ptr::write_bytes(p, 0, l.size());
        }
        p
    }

    #[inline]
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new_size: usize) -> *mut u8 {
        let nl = Layout::from_size_align_unchecked(new_size, l.align());
        match (is_small(&l), is_small(&nl)) {
            (false, false) => System.realloc(p, l, new_size),
            (true, true) if class_of(l.size()) == class_of(new_size) => p,
            _ => {
                let q = self.alloc(nl);
                if !q.is_null() {
                    std::ptr::copy_nonoverlapping(p, q, l.size().min(new_size));
                    self.dealloc(p, l);
                }
                q
            }
        }
    }
}
