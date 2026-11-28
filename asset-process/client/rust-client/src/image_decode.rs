//! Native image decoding for cache title artwork (fe.ag / ImageIO).
use crate::{sprite_pixels::Pixels, Error, Result};

#[cfg(any(not(windows), feature = "offscreen"))]
pub fn decode(_bytes: &[u8], _max_pixels: usize) -> Result<Pixels> {
    Err(Error::Unsupported("native image decoder on this platform"))
}

#[cfg(all(windows, not(feature = "offscreen")))]
pub fn decode(bytes: &[u8], max_pixels: usize) -> Result<Pixels> {
    windows::decode(bytes, max_pixels)
}

#[cfg(all(windows, not(feature = "offscreen")))]
mod windows {
    use super::*;
    use std::{ffi::c_void, ptr};
    #[repr(C)]
    struct Startup {
        version: u32,
        callback: *mut c_void,
        background: i32,
        codecs: i32,
    }
    #[repr(C)]
    struct Rect {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct BitmapData {
        width: u32,
        height: u32,
        stride: i32,
        format: i32,
        pixels: *mut c_void,
        reserved: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }
    struct Library(usize);
    impl Library {
        fn open(name: &str) -> Result<Self> {
            let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
            let handle = unsafe { LoadLibraryW(name.as_ptr()) };
            if handle.is_null() {
                Err(std::io::Error::last_os_error().into())
            } else {
                Ok(Self(handle as usize))
            }
        }
        fn symbol(&self, name: &[u8]) -> Result<*mut c_void> {
            let address = unsafe { GetProcAddress(self.0 as *mut c_void, name.as_ptr()) };
            if address.is_null() {
                Err(Error::Unsupported("Windows image decoder entry point"))
            } else {
                Ok(address)
            }
        }
    }
    impl Drop for Library {
        fn drop(&mut self) {
            unsafe {
                FreeLibrary(self.0 as *mut c_void);
            }
        }
    }
    macro_rules! api {
        ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
            #[allow(non_snake_case)]
            struct Api { $($name: unsafe extern "system" fn($($ty),*) -> $ret,)* _gdiplus: Library, _shell: Library }
            impl Api {
                fn load() -> Result<Self> {
                    // Absolute paths prevent application-directory DLL substitution.
                    let system = std::env::var_os("SystemRoot").ok_or(Error::Invalid("Windows system root"))?;
                    let directory = std::path::Path::new(&system).join("System32");
                    let gdiplus = Library::open(&directory.join("gdiplus.dll").to_string_lossy())?;
                    let shell = Library::open(&directory.join("shlwapi.dll").to_string_lossy())?;
                    Ok(Self { $($name: unsafe { std::mem::transmute(if stringify!($name) == "SHCreateMemStream" { shell.symbol(concat!(stringify!($name), "\0").as_bytes())? } else { gdiplus.symbol(concat!(stringify!($name), "\0").as_bytes())? }) },)* _gdiplus: gdiplus, _shell: shell })
                }
            }
            $(#[allow(non_snake_case)]
            unsafe fn $name($($arg: $ty),*) -> $ret { (api().expect("initialized image API").$name)($($arg),*) })*
        }
    }
    api! {
        GdiplusStartup(token: *mut usize, input: *const Startup, output: *mut c_void) -> i32;
        GdiplusShutdown(token: usize) -> ();
        GdipCreateBitmapFromStream(stream: *mut c_void, bitmap: *mut *mut c_void) -> i32;
        GdipGetImageWidth(image: *mut c_void, width: *mut u32) -> i32;
        GdipGetImageHeight(image: *mut c_void, height: *mut u32) -> i32;
        GdipBitmapLockBits(image: *mut c_void, rect: *const Rect, flags: u32, format: i32, data: *mut BitmapData) -> i32;
        GdipBitmapUnlockBits(image: *mut c_void, data: *mut BitmapData) -> i32;
        GdipDisposeImage(image: *mut c_void) -> i32;
        SHCreateMemStream(data: *const u8, size: u32) -> *mut c_void;
    }
    fn api() -> Result<&'static Api> {
        static API: std::sync::OnceLock<Result<Api>> = std::sync::OnceLock::new();
        API.get_or_init(Api::load).as_ref().map_err(Clone::clone)
    }
    // The stream must outlive the image; the image must outlive its pixel lock.
    struct Runtime(usize);
    impl Drop for Runtime {
        fn drop(&mut self) {
            unsafe { GdiplusShutdown(self.0) }
        }
    }
    struct Stream(*mut c_void);
    impl Drop for Stream {
        fn drop(&mut self) {
            unsafe {
                let table = *(self.0 as *const *const *const c_void);
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    std::mem::transmute(*table.add(2));
                release(self.0);
            }
        }
    }
    struct Image(*mut c_void);
    impl Drop for Image {
        fn drop(&mut self) {
            unsafe {
                GdipDisposeImage(self.0);
            }
        }
    }
    struct Lock<'a> {
        image: &'a Image,
        data: BitmapData,
    }
    impl Drop for Lock<'_> {
        fn drop(&mut self) {
            unsafe {
                GdipBitmapUnlockBits(self.image.0, &mut self.data);
            }
        }
    }
    fn check(status: i32) -> Result<()> {
        if status == 0 {
            Ok(())
        } else {
            Err(Error::Io(format!("GDI+ image status {status}")))
        }
    }
    pub fn decode(bytes: &[u8], max_pixels: usize) -> Result<Pixels> {
        api()?;
        let length =
            u32::try_from(bytes.len()).map_err(|_| Error::Invalid("encoded image size"))?;
        if length == 0 {
            return Err(Error::Invalid("empty encoded image"));
        }
        unsafe {
            let mut token = 0;
            check(GdiplusStartup(
                &mut token,
                &Startup {
                    version: 1,
                    callback: ptr::null_mut(),
                    background: 0,
                    codecs: 0,
                },
                ptr::null_mut(),
            ))?;
            let _runtime = Runtime(token);
            let stream = SHCreateMemStream(bytes.as_ptr(), length);
            if stream.is_null() {
                return Err(Error::Invalid("image stream allocation"));
            }
            let stream = Stream(stream);
            let mut bitmap = ptr::null_mut();
            check(GdipCreateBitmapFromStream(stream.0, &mut bitmap))?;
            if bitmap.is_null() {
                return Err(Error::Invalid("image allocation"));
            }
            let bitmap = Image(bitmap);
            let (mut width, mut height) = (0, 0);
            check(GdipGetImageWidth(bitmap.0, &mut width))?;
            check(GdipGetImageHeight(bitmap.0, &mut height))?;
            let count = (width as usize)
                .checked_mul(height as usize)
                .ok_or(Error::Invalid("image dimensions"))?;
            if count == 0
                || count > max_pixels
                || width > i32::MAX as u32
                || height > i32::MAX as u32
            {
                return Err(Error::Invalid("image dimensions"));
            }
            let mut data = BitmapData::default();
            check(GdipBitmapLockBits(
                bitmap.0,
                &Rect {
                    x: 0,
                    y: 0,
                    width: width as i32,
                    height: height as i32,
                },
                1,
                0x26200a,
                &mut data,
            ))?;
            let lock = Lock {
                image: &bitmap,
                data,
            };
            if lock.data.pixels.is_null()
                || lock.data.stride.unsigned_abs() as usize / 4 < width as usize
            {
                return Err(Error::Invalid("decoded image stride"));
            }
            let mut pixels = Vec::with_capacity(count);
            for y in 0..height as isize {
                let row = (lock.data.pixels as *const u8).offset(y * lock.data.stride as isize);
                for x in 0..width as usize {
                    // Keep ARGB, including opaque black, as Java PixelGrabber does.
                    pixels.push(ptr::read_unaligned(row.add(x * 4) as *const u32));
                }
            }
            Ok(Pixels {
                canvas: [width as usize, height as usize],
                offset: [0, 0],
                size: [width as usize, height as usize],
                pixels,
            })
        }
    }
}
