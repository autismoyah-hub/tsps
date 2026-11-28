//! Bounded UTF-16 clipboard reads, performed only for an explicit paste gesture.
#[cfg(any(not(windows), feature = "offscreen"))]
pub fn read(_: usize) -> Option<Vec<u16>> {
    None
}

#[cfg(all(windows, not(feature = "offscreen")))]
pub fn read(limit: usize) -> Option<Vec<u16>> {
    use std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn OpenClipboard(window: isize) -> i32;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(format: u32) -> isize;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalLock(handle: isize) -> *mut c_void;
        fn GlobalUnlock(handle: isize) -> i32;
        fn GlobalSize(handle: isize) -> usize;
    }
    struct Clipboard;
    impl Drop for Clipboard {
        fn drop(&mut self) {
            unsafe {
                CloseClipboard();
            }
        }
    }
    struct Lock(isize);
    impl Drop for Lock {
        fn drop(&mut self) {
            unsafe {
                GlobalUnlock(self.0);
            }
        }
    }
    unsafe {
        if limit == 0 {
            return Some(Vec::new());
        }
        if OpenClipboard(0) == 0 {
            return None;
        }
        let _clipboard = Clipboard;
        let handle = GetClipboardData(13); // CF_UNICODETEXT
        if handle == 0 {
            return None;
        }
        let size = GlobalSize(handle) / 2;
        let pointer = GlobalLock(handle).cast::<u16>();
        if pointer.is_null() {
            return None;
        }
        let _lock = Lock(handle);
        let text = std::slice::from_raw_parts(pointer, size.min(limit));
        Some(text.iter().copied().take_while(|&ch| ch != 0).collect())
    }
}

// Clipboard writes are performed only for an explicit copy/cut gesture.
#[cfg(any(not(windows), feature = "offscreen"))]
pub fn write(_: &[u16]) -> bool {
    false
}
#[cfg(all(windows, not(feature = "offscreen")))]
pub fn write(text: &[u16]) -> bool {
    use std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn OpenClipboard(window: isize) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(format: u32, handle: isize) -> isize;
        fn GetActiveWindow() -> isize;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalAlloc(flags: u32, size: usize) -> isize;
        fn GlobalFree(handle: isize) -> isize;
        fn GlobalLock(handle: isize) -> *mut c_void;
        fn GlobalUnlock(handle: isize) -> i32;
    }
    let Some(bytes) = text.len().checked_add(1).and_then(|n| n.checked_mul(2)) else {
        return false;
    };
    unsafe {
        let handle = GlobalAlloc(2, bytes); // GMEM_MOVEABLE; ownership passes on successful SetClipboardData.
        if handle == 0 {
            return false;
        }
        let pointer = GlobalLock(handle).cast::<u16>();
        if pointer.is_null() {
            GlobalFree(handle);
            return false;
        }
        std::ptr::copy_nonoverlapping(text.as_ptr(), pointer, text.len());
        *pointer.add(text.len()) = 0;
        GlobalUnlock(handle);
        if OpenClipboard(GetActiveWindow()) == 0 {
            GlobalFree(handle);
            return false;
        }
        let success = EmptyClipboard() != 0 && SetClipboardData(13, handle) != 0;
        CloseClipboard();
        if !success {
            GlobalFree(handle);
        }
        success
    }
}
