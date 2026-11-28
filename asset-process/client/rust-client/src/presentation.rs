//! Completed-frame GDI presentation. All resources and callbacks belong to the
//! window's UI thread. No rendering pixels, layout rules or input are changed.
use std::{ffi::c_void, ptr};
type H = isize;
#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Paint {
    dc: H,
    erase: i32,
    rect: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}
#[repr(C)]
struct Bitmap {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    image_size: u32,
    xppm: i32,
    yppm: i32,
    used: u32,
    important: u32,
}
impl Bitmap {
    fn rgb(size: [i32; 2]) -> Self {
        Self {
            size: 40,
            width: size[0],
            height: -size[1],
            planes: 1,
            bits: 32,
            compression: 0,
            image_size: 0,
            xppm: 0,
            yppm: 0,
            used: 0,
            important: 0,
        }
    }
}
#[link(name = "user32")]
unsafe extern "system" {
    fn GetDC(hwnd: H) -> H;
    fn ReleaseDC(hwnd: H, dc: H) -> i32;
    fn BeginPaint(hwnd: H, paint: *mut Paint) -> H;
    fn EndPaint(hwnd: H, paint: *const Paint) -> i32;
    fn GetClientRect(hwnd: H, rect: *mut Rect) -> i32;
    fn GetWindowLongPtrW(hwnd: H, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: H, index: i32, value: isize) -> isize;
    fn InvalidateRect(hwnd: H, rect: *const Rect, erase: i32) -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: H) -> H;
    fn CreateDIBSection(
        dc: H,
        info: *const Bitmap,
        usage: u32,
        bits: *mut *mut c_void,
        section: H,
        offset: u32,
    ) -> H;
    fn SelectObject(dc: H, object: H) -> H;
    fn DeleteObject(object: H) -> i32;
    fn DeleteDC(dc: H) -> i32;
    fn PatBlt(dc: H, x: i32, y: i32, w: i32, h: i32, rop: u32) -> i32;
    fn BitBlt(dst: H, x: i32, y: i32, w: i32, h: i32, src: H, sx: i32, sy: i32, rop: u32) -> i32;
    fn StretchBlt(
        dst: H,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        src: H,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        rop: u32,
    ) -> i32;
    fn StretchDIBits(
        dc: H,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        bits: *const c_void,
        info: *const Bitmap,
        usage: u32,
        rop: u32,
    ) -> i32;
    fn GdiFlush() -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetLastError(error: u32);
    fn GetLastError() -> u32;
}
const COPY: u32 = 0x00cc0020;
struct Surface {
    dc: H,
    bitmap: H,
    previous: H,
    bits: *mut c_void,
    size: [i32; 2],
}
impl Surface {
    unsafe fn new(device: H, size: [i32; 2]) -> std::io::Result<Self> {
        if size[0] <= 0 || size[1] <= 0 || i64::from(size[0]) * i64::from(size[1]) > 32_000_000 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "presentation surface size",
            ));
        }
        let dc = unsafe { CreateCompatibleDC(device) };
        if dc == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut bits = ptr::null_mut();
        let bitmap = unsafe { CreateDIBSection(device, &Bitmap::rgb(size), 0, &mut bits, 0, 0) };
        if bitmap == 0 || bits.is_null() {
            let error = std::io::Error::last_os_error();
            unsafe {
                if bitmap != 0 {
                    DeleteObject(bitmap);
                }
                DeleteDC(dc);
            }
            return Err(error);
        }
        let previous = unsafe { SelectObject(dc, bitmap) };
        if previous == 0 || previous == -1 {
            unsafe {
                DeleteObject(bitmap);
                DeleteDC(dc)
            };
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            dc,
            bitmap,
            previous,
            bits,
            size,
        })
    }
    unsafe fn copy(&self, dc: H, size: [i32; 2]) -> bool {
        let copied = if self.size == size {
            unsafe { BitBlt(dc, 0, 0, size[0], size[1], self.dc, 0, 0, COPY) != 0 }
        } else {
            unsafe {
                StretchBlt(
                    dc,
                    0,
                    0,
                    size[0],
                    size[1],
                    self.dc,
                    0,
                    0,
                    self.size[0],
                    self.size[1],
                    COPY,
                ) != 0
            }
        };
        copied || unsafe { self.copy_dib(dc, size) }
    }
    /// Some window devices reject DC-to-DC blits. Replay the same completed pixels.
    unsafe fn copy_dib(&self, dc: H, size: [i32; 2]) -> bool {
        // CreateDIBSection requires synchronization before reading its pixel pointer.
        if unsafe { GdiFlush() } == 0 {
            return false;
        }
        unsafe {
            StretchDIBits(
                dc,
                0,
                0,
                size[0],
                size[1],
                0,
                0,
                self.size[0],
                self.size[1],
                self.bits,
                &Bitmap::rgb(self.size),
                0,
                COPY,
            ) > 0
        }
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            GdiFlush();
            SelectObject(self.dc, self.previous);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}
#[derive(Default)]
struct State {
    front: Option<Surface>,
    back: Option<Surface>,
    #[cfg(feature="native-profiling")]
    events: Option<std::cell::Cell<crate::presentation_trace::Events>>,
}
pub struct Presenter {
    hwnd: H,
    state: Box<State>,
    _ui_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl Presenter {
    /// The completed physical surface, including letterboxing; not desktop readback.
    #[cfg(feature="native-profiling")]
    pub fn completed_frame(&self)->crate::Result<crate::raster::Raster> {
        let front=self.state.front.as_ref().ok_or(crate::Error::Invalid("no completed presentation frame"))?;
        let [width,height]=front.size;
        let count=i64::from(width)*i64::from(height);
        if width<=0||height<=0||count>16_588_800 {return Err(crate::Error::Invalid("presentation capture dimensions"));}
        if unsafe {GdiFlush()}==0 {return Err(std::io::Error::last_os_error().into());}
        // GDI owns the alpha byte. Canonicalize it without changing visible RGB.
        let pixels=unsafe {std::slice::from_raw_parts(front.bits.cast::<u32>(),count as usize)}
            .iter().map(|&pixel|pixel|0xff000000).collect();
        crate::raster::Raster::from_pixels(width as usize,height as usize,pixels)
    }
    /// PNG (benchmark captures are PNG-only), create_new: an existing file is never replaced.
    #[cfg(feature="native-profiling")]
    pub fn save_completed_frame(&self,path:&std::path::Path)->crate::Result<()> {
        crate::study_profile::write_png_new(path,&self.completed_frame()?)
    }
    #[cfg(feature="native-profiling")]
    pub fn track_events(&mut self) {self.state.events=Some(std::cell::Cell::new(Default::default()));}
    #[cfg(feature="native-profiling")]
    pub fn events(&self)->crate::presentation_trace::Events {self.state.events.as_ref().map_or_else(Default::default,std::cell::Cell::get)}
    /// The window must live on this thread and its GWLP_USERDATA must be unused.
    pub unsafe fn new(hwnd: H) -> std::io::Result<Self> {
        if unsafe { GetWindowLongPtrW(hwnd, -21) } != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "window userdata already occupied",
            ));
        }
        let mut state = Box::<State>::default();
        unsafe {
            SetLastError(0);
            SetWindowLongPtrW(hwnd, -21, (&mut *state as *mut State) as isize);
        }
        let error = unsafe { GetLastError() };
        if error != 0 {
            return Err(std::io::Error::from_raw_os_error(error as i32));
        }
        Ok(Self {
            hwnd,
            state,
            _ui_thread: std::marker::PhantomData,
        })
    }
    pub fn submit(
        &mut self,
        pixels: &[u32],
        logical: [i32; 2],
        physical: [i32; 2],
        origin: [i32; 2],
        size: [i32; 2],
    ) -> std::io::Result<()> {
        if logical[0] <= 0
            || logical[1] <= 0
            || logical[0] as usize * logical[1] as usize != pixels.len()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "incomplete presentation framebuffer",
            ));
        }
        let dc = unsafe { GetDC(self.hwnd) };
        if dc == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let result = (|| {
            if self.state.back.as_ref().is_none_or(|b| b.size != physical) {
                self.state.back = Some(unsafe { Surface::new(dc, physical) }?);
            }
            let back = self.state.back.as_ref().unwrap();
            let bitmap = Bitmap {
                size: 40,
                width: logical[0],
                height: -logical[1],
                planes: 1,
                bits: 32,
                compression: 0,
                image_size: 0,
                xppm: 0,
                yppm: 0,
                used: 0,
                important: 0,
            };
            // Letterbox and framebuffer composition affect only the private DC.
            let clear = unsafe { PatBlt(back.dc, 0, 0, physical[0], physical[1], 0x42) };
            let draw = unsafe {
                StretchDIBits(
                    back.dc,
                    origin[0],
                    origin[1],
                    size[0],
                    size[1],
                    0,
                    0,
                    logical[0],
                    logical[1],
                    pixels.as_ptr().cast(),
                    &bitmap,
                    0,
                    COPY,
                )
            };
            let flushed = unsafe { GdiFlush() };
            if clear == 0 || draw == 0 || draw == -1 || flushed == 0 {
                return Err(std::io::Error::last_os_error());
            }
            // Publish only a fully composed frame. WM_PAINT can replay this frame
            // while native move/size modal loops suspend the normal render loop.
            std::mem::swap(&mut self.state.front, &mut self.state.back);
            if !unsafe { self.state.front.as_ref().unwrap().copy(dc, physical) }
                || unsafe { GdiFlush() } == 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        })();
        unsafe {
            ReleaseDC(self.hwnd, dc);
        }
        if result.is_err() {
            unsafe {
                InvalidateRect(self.hwnd, ptr::null(), 0);
            }
        }
        result
    }
}
impl Drop for Presenter {
    fn drop(&mut self) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, -21, 0);
        }
    }
}
/// Called only by the owning window procedure. Never borrows State across a
/// message dispatch in submit, and never paints from a partially composed back DC.
pub unsafe fn paint(hwnd: H, message: u32) -> Option<isize> {
    #[cfg(feature="native-profiling")]
    if matches!(message,0x0f|0x14|5) {
        let state=unsafe {GetWindowLongPtrW(hwnd,-21)} as *const State;
        if let Some(events)=unsafe {state.as_ref()}.and_then(|state|state.events.as_ref()) {
            let mut value=events.get();value.event(message);events.set(value);
        }
    }
    if message != 0x0f {
        return None;
    }
    let state = unsafe { GetWindowLongPtrW(hwnd, -21) } as *const State;
    if state.is_null() {
        return None;
    }
    let mut paint: Paint = unsafe { std::mem::zeroed() };
    let dc = unsafe { BeginPaint(hwnd, &mut paint) };
    if dc != 0 {
        let mut rect: Rect = unsafe { std::mem::zeroed() };
        if unsafe { GetClientRect(hwnd, &mut rect) } != 0 {
            if let Some(front) = unsafe { &*state }.front.as_ref() {
                unsafe {
                    front.copy(dc, [rect.right, rect.bottom]);
                    GdiFlush();
                }
            }
        }
    }
    unsafe {
        EndPaint(hwnd, &paint);
    }
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: H,
            menu: H,
            instance: H,
            parameter: *mut c_void,
        ) -> H;
        fn DestroyWindow(hwnd: H) -> i32;
        fn GetGuiResources(process: H, flags: u32) -> u32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> H;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn GetPixel(dc: H, x: i32, y: i32) -> u32;
    }
    struct Window(H);
    impl Drop for Window {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    fn window() -> Window {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                class.as_ptr(),
                0,
                -10000,
                -10000,
                64,
                64,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };
        assert_ne!(hwnd, 0);
        Window(hwnd)
    }

    #[test]
    fn completed_frame_survives_failed_submit_and_repaint() {
        let window = window();
        let mut presenter = unsafe { Presenter::new(window.0) }.unwrap();
        assert!(
            unsafe { Presenter::new(window.0) }.is_err(),
            "userdata ownership"
        );
        let red = [0xffff0000; 4];
        presenter
            .submit(&red, [2, 2], [64, 64], [4, 4], [16, 16])
            .unwrap();
        let front = presenter.state.front.as_ref().unwrap().dc;
        assert_eq!(unsafe { GetPixel(front, 4, 4) }, 0x000000ff);
        assert_eq!(unsafe { GetPixel(front, 0, 0) }, 0);
        assert!(presenter
            .submit(&red[..3], [2, 2], [64, 64], [0, 0], [64, 64])
            .is_err());
        assert!(presenter
            .submit(&red, [2, 2], [0, 0], [0, 0], [0, 0])
            .is_err());
        assert_eq!(presenter.state.front.as_ref().unwrap().dc, front);
        unsafe {
            InvalidateRect(window.0, ptr::null(), 0);
        }
        assert_eq!(unsafe { paint(window.0, 0x0f) }, Some(0));
        assert_eq!(unsafe { GetPixel(front, 4, 4) }, 0x000000ff);
        presenter
            .submit(&[0xff0000ff; 4], [2, 2], [32, 32], [0, 0], [32, 32])
            .unwrap();
        assert_eq!(
            unsafe { GetPixel(presenter.state.front.as_ref().unwrap().dc, 0, 0) },
            0x00ff0000
        );
        assert_eq!(
            red, [0xffff0000; 4],
            "presentation never edits input pixels"
        );
        drop(presenter);
        assert_eq!(unsafe { GetWindowLongPtrW(window.0, -21) }, 0);
        assert_eq!(unsafe { paint(window.0, 0x0f) }, None);
    }

    #[cfg(feature="native-profiling")]
    #[test]
    fn trace_observes_window_local_events_and_successful_and_failed_submissions() {
        use crate::presentation_trace::{Trace,Sample,Events};
        let first=window();let second=window();
        let mut a=unsafe {Presenter::new(first.0)}.unwrap();let b=unsafe {Presenter::new(second.0)}.unwrap();
        a.track_events();
        unsafe {paint(first.0,5);paint(first.0,0x14);paint(second.0,5);}
        assert_eq!(a.events(),Events{paint:0,erase:1,resize:1});assert_eq!(b.events(),Events::default());
        let path=std::env::temp_dir().join(format!("native-present-trace-{}.csv",std::process::id()));
        let mut trace=Some(Trace::new(&path).unwrap());
        for pixels in [&[0xff123456u32;4][..],&[0xff123456u32;3][..]] {
            let start=std::time::Instant::now();let success=a.submit(pixels,[2,2],[64,64],[0,0],[64,64]).is_ok();
            let done=std::time::Instant::now();
            assert!(Trace::complete(&mut trace,Sample{times:[start,start,start,start,start,done],logical:[2,2],physical:[64,64],pixels,success,events:a.events()}).is_none());
        }
        unsafe {paint(first.0,0x0f);}
        assert_eq!(a.events().paint,1);assert_eq!(b.events(),Events::default());
        drop(trace);let text=std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(),3);assert!(text.lines().nth(1).unwrap().contains(",true,"));assert!(text.lines().nth(2).unwrap().contains(",false,"));
        std::fs::remove_file(path).unwrap();
    }

    #[cfg(feature="native-profiling")]
    #[test]
    fn completed_capture_includes_letterbox_and_preserves_prior_output() {
        let window=window();let mut presenter=unsafe {Presenter::new(window.0)}.unwrap();
        assert!(presenter.completed_frame().is_err());
        let pixels=[0xffff0000,0xff00ff00,0xff0000ff,0xffffffff];
        presenter.submit(&pixels,[2,2],[8,8],[2,2],[4,4]).unwrap();
        let first=presenter.completed_frame().unwrap();
        assert_eq!(first.pixels().len(),64);
        for (x,y,color) in [(0,0,0xff000000),(2,2,0xffff0000),(5,2,0xff00ff00),(2,5,0xff0000ff),(5,5,0xffffffff),(7,7,0xff000000)] {
            assert_eq!(first.pixels()[y*8+x],color);
        }
        assert!(presenter.submit(&pixels[..3],[2,2],[8,8],[0,0],[8,8]).is_err());
        assert_eq!(presenter.completed_frame().unwrap().pixels(),first.pixels());
        let path=std::env::temp_dir().join(format!("native-completed-capture-{}.png",std::process::id()));
        presenter.save_completed_frame(&path).unwrap();let before=std::fs::read(&path).unwrap();
        assert_eq!(before,first.png().unwrap());
        assert_eq!(&before[..8],b"\x89PNG\r\n\x1a\n");assert_eq!(&before[12..24],b"IHDR\0\0\0\x08\0\0\0\x08");
        presenter.submit(&[0xff123456],[1,1],[4,4],[0,0],[4,4]).unwrap();
        assert!(presenter.save_completed_frame(&path).is_err());assert_eq!(std::fs::read(&path).unwrap(),before);
        assert_eq!(presenter.completed_frame().unwrap().pixels(),&[0xff123456;16]);
        std::fs::remove_file(path).unwrap();
    }

    const GDI_CHILD: &str = "GIELINOR_PRESENTATION_GDI_CHILD";
    /// GetGuiResources counts the whole process. The parent re-runs this test
    /// alone in a child test process so no parallel test thread can move it.
    #[test]
    fn repeated_resize_and_drop_release_gdi_surfaces() {
        if std::env::var_os(GDI_CHILD).is_none() {
            let name = concat!(module_path!(), "::repeated_resize_and_drop_release_gdi_surfaces");
            let name = name.split_once("::").unwrap().1;
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([name, "--exact", "--test-threads=1", "--nocapture"])
                .env(GDI_CHILD, "1")
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                output.status.success() && stdout.contains("1 passed"),
                "isolated GDI child failed: {stdout}{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let window = window();
        // Warm GDI lazy initialization before counting owned resources.
        {
            let mut p = unsafe { Presenter::new(window.0) }.unwrap();
            p.submit(&[0xff000000], [1, 1], [8, 8], [0, 0], [8, 8])
                .unwrap();
        }
        let process = unsafe { GetCurrentProcess() };
        let before = unsafe { GetGuiResources(process, 0) };
        for _ in 0..16 {
            let mut p = unsafe { Presenter::new(window.0) }.unwrap();
            for size in [8, 32, 16, 64, 8] {
                p.submit(&[0xff123456], [1, 1], [size, size], [0, 0], [size, size])
                    .unwrap();
            }
        }
        assert_eq!(unsafe { GetGuiResources(process, 0) }, before);
        assert_eq!(unsafe { GetWindowLongPtrW(window.0, -21) }, 0);
    }

    #[test]
    fn dib_copy_preserves_top_down_pixels_when_scaled() {
        let window = window();
        let dc = unsafe { GetDC(window.0) };
        assert_ne!(dc, 0);
        let source = unsafe { Surface::new(dc, [2, 2]) }.unwrap();
        let target = unsafe { Surface::new(dc, [4, 4]) }.unwrap();
        unsafe {
            assert_ne!(GdiFlush(), 0);
            std::ptr::copy_nonoverlapping(
                [0xffff0000u32, 0xff00ff00, 0xff0000ff, 0xffffffff].as_ptr(),
                source.bits.cast::<u32>(),
                4,
            );
            assert!(source.copy_dib(target.dc, [4, 4]));
            assert_ne!(GdiFlush(), 0);
            for (x, y, color) in [
                (0, 0, 0x000000ff),
                (3, 0, 0x0000ff00),
                (0, 3, 0x00ff0000),
                (3, 3, 0x00ffffff),
            ] {
                assert_eq!(GetPixel(target.dc, x, y), color);
            }
            ReleaseDC(window.0, dc);
        }
    }
}
