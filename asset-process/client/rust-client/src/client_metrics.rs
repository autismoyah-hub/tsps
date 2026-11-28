//! Native render timing and revision-239 server ping replies.
use crate::{buffer::Transform, protocol::Packet, writer::Writer};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
static HEAP_BYTES: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature="native-profiling")]
static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature="native-profiling")]
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature="native-profiling")]
fn record_allocation(size:usize) {
    ALLOC_CALLS.fetch_add(1,Ordering::Relaxed);
    REQUESTED_BYTES.fetch_add(size,Ordering::Relaxed);
}
/// Process-wide successful allocation/reallocation calls and requested sizes.
/// Reallocations count their requested new size, not only retained growth.
#[cfg(feature="native-profiling")]
pub fn allocation_snapshot()->[usize;3] {
    [HEAP_BYTES.load(Ordering::Relaxed),ALLOC_CALLS.load(Ordering::Relaxed),REQUESTED_BYTES.load(Ordering::Relaxed)]
}
/// Tracks native Rust allocations for the client memory display, excluding OS graphics/audio buffers.
pub struct TrackedAllocator;
unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            HEAP_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
            #[cfg(feature="native-profiling")]
            record_allocation(layout.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            HEAP_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
            #[cfg(feature="native-profiling")]
            record_allocation(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(p, layout);
        }
        HEAP_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(p, layout, size) };
        if !result.is_null() {
            #[cfg(feature="native-profiling")]
            record_allocation(size);
            if size >= layout.size() {
                HEAP_BYTES.fetch_add(size - layout.size(), Ordering::Relaxed);
            } else {
                HEAP_BYTES.fetch_sub(layout.size() - size, Ordering::Relaxed);
            }
        }
        result
    }
}
pub fn heap_kib() -> i32 {
    (HEAP_BYTES.load(Ordering::Relaxed) / 1024) as i32
}

#[derive(Clone, Copy)]
pub struct Display {
    pub fps: i32,
    pub memory_kib: i32,
    pub low_detail: bool,
}
impl Display {
    pub fn draw(
        &self,
        font: &crate::font::Font,
        raster: &mut crate::raster::Raster,
        origin: [i32; 2],
        width: i32,
    ) -> crate::Result<()> {
        let lines = [
            (format!("Fps:{}", self.fps), 0xffff00),
            (
                format!("Mem:{}k", self.memory_kib),
                if self.memory_kib > 327680 && !self.low_detail {
                    0xff0000
                } else {
                    0xffff00
                },
            ),
        ];
        for (index, (text, color)) in lines.iter().enumerate() {
            font.draw(
                raster,
                text,
                origin[0] + width - font.metrics.width(text, &[]),
                origin[1] + 15 * (index as i32 + 1),
                crate::font::Style {
                    color: *color,
                    shadow: None,
                    alpha: 256,
                },
                &[],
            )?;
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct FrameRate {
    times: [u64; 32],
    next: usize,
    pub fps: i32,
}
impl FrameRate {
    pub fn frame(&mut self, now: u64) {
        let previous = self.times[self.next];
        self.times[self.next] = now;
        self.next = (self.next + 1) & 31;
        if previous != 0 && now > previous {
            let elapsed = now.wrapping_sub(previous) as i32;
            if elapsed != 0 {
                self.fps = ((elapsed >> 1).wrapping_add(32000)).wrapping_div(elapsed);
            }
        }
    }
}
pub fn ping_reply(challenge: [u32; 2], fps: i32, gc_percent: i32) -> Packet {
    let mut w = Writer::new();
    w.u32_middle(challenge[0]);
    w.u32(challenge[1]);
    w.transformed(fps as u8, Transform::Add);
    w.transformed(gc_percent as u8, Transform::Neg);
    Packet {
        opcode: 1,
        payload: w.into_vec(),
    }
}
