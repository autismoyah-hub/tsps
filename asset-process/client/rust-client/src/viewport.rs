//! Native Windows framebuffer window and offline scene navigation.
#[derive(Default)]
pub struct StudyOptions {
    pub smoke:bool,
    #[cfg(feature="native-profiling")] pub benchmark:Option<std::path::PathBuf>,
    pub start:Option<StudyViewpoint>,
    pub viewpoints:Vec<StudyViewpoint>,
    pub far:Option<i32>,
    #[cfg(feature="custom-assets")] pub cutaway:bool,
    #[cfg(feature="enhanced-renderer")] pub renderer:Option<crate::enhanced::Settings>,
}
#[derive(Clone,Copy,Debug)]
pub struct StudyViewpoint {pub world:[i32;2],pub distance:f32}
fn study_target(scene:&crate::scene::Scene,plane:usize,local:[f32;2])->[f32;3] {
    [local[0],scene.height_fine(plane,local[0] as i32,local[1] as i32).unwrap_or(0) as f32,local[1]]
}
fn study_viewpoint(base:[i32;2],dimensions:[usize;2],point:StudyViewpoint)->crate::Result<([f32;2],f32)> {
 if !point.distance.is_finite()||!(250.0..=12000.0).contains(&point.distance){return Err(crate::Error::Invalid("study viewpoint distance"));}
 let local=std::array::from_fn::<_,2,_>(|i|i64::from(point.world[i])-i64::from(base[i]));
 if (0..2).any(|i|local[i]<1||local[i]>dimensions[i].saturating_sub(2)as i64){return Err(crate::Error::Invalid("study viewpoint outside navigable scene"));}
 Ok((local.map(|v|v as f32*128.0),point.distance))
}
#[cfg(test)]
mod viewpoint_tests {
 use super::*;
 #[test]
 fn source_viewpoints_resolve_world_coordinates_and_reject_invalid_inputs() {
  let base=[10368,64];
  for (x,z,d) in [(53,46,2200.),(64,64,2200.),(81,56,2600.),(59,56,10500.)] {
   let p=StudyViewpoint{world:[base[0]+x,base[1]+z],distance:d};
   assert_eq!(study_viewpoint(base,[104,104],p).unwrap(),([x as f32*128.,z as f32*128.],d));
  }
  for distance in [f32::NAN,f32::INFINITY,249.,12001.] {assert!(study_viewpoint(base,[104,104],StudyViewpoint{world:[10400,96],distance}).is_err());}
  for world in [[10368,96],[10471,96],[10400,64],[10400,167],[i32::MAX,i32::MIN]] {assert!(study_viewpoint(base,[104,104],StudyViewpoint{world,distance:2200.}).is_err());}
 }
 #[test]
 fn upper_plane_study_target_uses_the_selected_scene_height() {
  use crate::{rebuild::Chunk,terrain::{Region,Tile}};
  let region=Region{tiles:vec![Tile::default();4*4096],extension_flags:0,height_rules:vec![Some(30);4*4096]};
  let mut scene=crate::scene::Scene::empty_sized(3200,3200,16,16).unwrap();
  for plane in 0..4 {
   scene.add_chunk(&region,Chunk{plane:plane as u8,rotation:0,zone_x:0,zone_y:0},plane,0,0).unwrap();
  }
  for plane in 0..4 {
   let target=study_target(&scene,plane,[320.,320.]);
   assert_eq!(target,[320.,-240.*(plane+1) as f32,320.],"selected plane {plane}");
   // Matches the original camera's plane-owned terrain focus, before follow-height offset.
   let mut camera=crate::camera::State::default();
   camera.focus=[320,0,320];
   camera.tick_follow(&scene,[3200*128+320;2],plane,0);
   assert_eq!(camera.focus,target.map(|v|v as i32));
  }
 }
}

fn study_smoke_finished(smoke:bool,total_frames:u64,presentation_errors:u64)->crate::Result<bool> {
    if !smoke || total_frames<2 {return Ok(false);}
    if presentation_errors!=0 {return Err(crate::Error::Invalid("offline smoke presentation failed"));}
    Ok(true)
}
#[cfg(test)]
mod study_tests {
    use super::*;
    #[test]
    fn smoke_completion_is_independent_of_fps_resets_and_rejects_failed_presentations() {
        assert!(!study_smoke_finished(true,1,0).unwrap());
        // The FPS counter may reset after every slow frame; total frames do not.
        assert!(study_smoke_finished(true,2,0).unwrap());
        assert!(study_smoke_finished(true,3,0).unwrap());
        assert!(study_smoke_finished(true,2,1).is_err());
        assert!(!study_smoke_finished(false,100,1).unwrap());
    }
}
#[cfg(windows)]
mod windows {
    use super::{StudyOptions,study_smoke_finished,study_viewpoint,study_target};
    use crate::{
        disk_cache::DiskCache,
        floor,
        rebuild::Normal,
        render3d::{Camera, Frame},
        scene::Scene,
        texture::{Definition, Texture},
        world_mesh::WorldMesh,
        Error, Result,
    };
    use std::{
        collections::HashMap,
        ffi::c_void,
        ptr,
        time::{Duration, Instant},
    };
    #[cfg(not(feature = "offscreen"))]
    fn mouse_button(physical: u8) -> u8 {
        // GetKeyState observes modifiers at this queued mouse event, including
        // Alt presses consumed by a focused widget editor.
        // https://learn.microsoft.com/windows/win32/inputdev/wm-lbuttondown
        crate::input::mouse_button(physical, unsafe { GetKeyState(18) } < 0, false)
    }

    #[cfg(not(feature = "offscreen"))]
    #[test]
    fn original_win32_modifier_mapping_uses_thread_keyboard_state() -> Result<()> {
        // SetKeyboardState changes only this test thread, not desktop input.
        // https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setkeyboardstate
        struct Restore([u8; 256]);
        impl Drop for Restore {
            fn drop(&mut self) { unsafe { SetKeyboardState(self.0.as_ptr()); } }
        }
        let mut original = [0; 256];
        assert_ne!(unsafe { GetKeyboardState(original.as_mut_ptr()) }, 0);
        let _restore = Restore(original);
        let mut r = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/mouse-modifier-click-jvm.bin"));
        assert_eq!(r.u32()?, 96);
        let mut checked = 0;
        for _ in 0..96 {
            let _camera = r.u32()?;
            let physical = r.u32()? as u8;
            let modifiers = r.u32()?;
            let expected = r.u32()? as u8;
            let length = r.u32()? as i32;
            if length >= 0 { r.bytes(length as usize)?; }
            // AWT Meta is not a separate Windows keyboard mouse modifier.
            if modifiers & 8 != 0 { continue; }
            let mut state = original;
            for (key, mask) in [(16, 1), (17, 2), (18, 4)] {
                state[key] = if modifiers & mask != 0 { 128 } else { 0 };
            }
            assert_ne!(unsafe { SetKeyboardState(state.as_ptr()) }, 0);
            assert_eq!(1 << mouse_button(physical), expected, "physical{physical} modifiers{modifiers}");
            checked += 1;
        }
        assert_eq!(checked, 48);
        assert_eq!(r.remaining(), 0);
        Ok(())
    }

    #[cfg(not(feature = "offscreen"))]
    fn mouse_capture(window: Handle, down: bool, flags: usize) {
        // Windows AWT SetDragCapture/ReleaseDragCapture: preserve another
        // control's capture and use AWT's three primary-button release flags.
        // Logical Input.buttons can already be clear after releasing one button.
        unsafe {
            let owner = GetCapture();
            if down {
                if owner == 0 { SetCapture(window); }
            } else if owner == window && flags & (1 | 2 | 16) == 0 {
                ReleaseCapture();
            }
        }
    }

    #[cfg(not(feature = "offscreen"))]
    #[test]
    fn original_win32_drag_capture_ownership() -> Result<()> {
        // Hidden windows on this test thread; no desktop input or focus changes.
        // The fixture records the original client's installed Windows AWT backend.
        struct Owned([Handle; 2]);
        impl Drop for Owned {
            fn drop(&mut self) {
                unsafe {
                    if self.0.contains(&GetCapture()) { ReleaseCapture(); }
                    for window in self.0 { if window != 0 { DestroyWindow(window); } }
                }
            }
        }
        #[repr(C)]
        struct GuiInfo {
            size: u32, flags: u32,
            active: Handle, focus: Handle, capture: Handle,
            menu_owner: Handle, move_size: Handle, caret: Handle,
            rect: Rect,
        }
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetGUIThreadInfo(thread: u32, info: *mut GuiInfo) -> i32;
        }
        let mut foreground: GuiInfo = unsafe { std::mem::zeroed() };
        foreground.size = std::mem::size_of::<GuiInfo>() as u32;
        if unsafe { GetForegroundWindow() } != 0 {
            assert_ne!(unsafe { GetGUIThreadInfo(0, &mut foreground) }, 0);
            assert_eq!(foreground.capture, 0, "defer isolated capture test while foreground drag is active");
        }
        assert_eq!(unsafe { GetCapture() }, 0, "test thread already has capture");
        let mut owned = Owned([0; 2]);
        let class = wide("STATIC");
        let title = wide("Agent1 private capture regression");
        for window in &mut owned.0 {
            *window = unsafe { CreateWindowExW(0, class.as_ptr(), title.as_ptr(), 0,
                -10000, -10000, 100, 100, 0, 0, GetModuleHandleW(ptr::null()), ptr::null_mut()) };
            assert_ne!(*window, 0, "create hidden test window");
        }
        let mut r = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/native-mouse-capture-jvm.bin"));
        let count = r.u32()?;
        assert_eq!(count, 320);
        for step in 0..count {
            let target = r.u32()? as usize;
            let message = r.u32()?;
            let flags = r.u32()? as usize;
            let expected = r.u32()? as i32;
            let down = matches!(message, 0x201 | 0x204 | 0x207);
            assert!(down || matches!(message, 0x202 | 0x205 | 0x208));
            mouse_capture(owned.0[target], down, flags);
            let actual = unsafe { GetCapture() };
            assert!(actual == 0 || owned.0.contains(&actual), "unexpected capture owner");
            let actual = owned.0.iter().position(|&window| window == actual).map_or(-1, |v| v as i32);
            assert_eq!(actual, expected, "capture step{step} target{target} message{message:#x} flags{flags:#x}");
        }
        assert_eq!(r.remaining(), 0);
        let mut r = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/native-extra-mouse-jvm.bin"));
        assert_eq!(r.u32()?, 640);
        for step in 0..640 {
            let target=r.u32()? as usize;
            let message=r.u32()?;
            let flags=r.u32()? as usize;
            for _ in 0..4 { r.u32()?; } // AWT event/button and original held/pressed state.
            let expected=r.u32()? as i32;
            let (_,down)=crate::input::windows_mouse_button(message,flags).expect("original native button");
            mouse_capture(owned.0[target],down,flags);
            let actual=unsafe {GetCapture()};
            assert!(actual==0 || owned.0.contains(&actual));
            let actual=owned.0.iter().position(|&window|window==actual).map_or(-1,|v|v as i32);
            assert_eq!(actual,expected,"five-button capture step{step}");
        }
        assert_eq!(r.remaining(), 0);
        assert_eq!(unsafe { GetCapture() }, 0);
        Ok(())
    }

    type Handle = isize;
    #[cfg(not(feature = "offscreen"))]
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    // https://learn.microsoft.com/windows/win32/api/winuser/ns-winuser-trackmouseevent
    #[cfg(not(feature = "offscreen"))]
    #[repr(C)]
    struct TrackMouse { size: u32, flags: u32, window: Handle, hover_millis: u32 }
    #[cfg(not(feature = "offscreen"))]
    #[repr(C)]
    struct Msg {
        hwnd: Handle,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt: Point,
        private: u32,
    }
    #[cfg(not(feature = "offscreen"))]
    #[repr(C)]
    struct WndClass {
        size: u32,
        style: u32,
        proc: Option<unsafe extern "system" fn(Handle, u32, usize, isize) -> isize>,
        class_extra: i32,
        window_extra: i32,
        instance: Handle,
        icon: Handle,
        cursor: Handle,
        background: Handle,
        menu: *const u16,
        name: *const u16,
        icon_small: Handle,
    }
    #[cfg(not(feature = "offscreen"))]
    #[link(name = "user32")]
    unsafe extern "system" {
        fn RegisterClassExW(c: *const WndClass) -> u16;
        fn TrackMouseEvent(track: *mut TrackMouse) -> i32;
        fn GetKeyState(key: i32) -> i16;
        fn GetCapture() -> Handle;
        fn SetCapture(window: Handle) -> Handle;
        fn ReleaseCapture() -> i32;
        #[cfg(test)]
        fn GetKeyboardState(state: *mut u8) -> i32;
        #[cfg(test)]
        fn SetKeyboardState(state: *const u8) -> i32;
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Handle,
            menu: Handle,
            instance: Handle,
            param: *mut c_void,
        ) -> Handle;
        fn DefWindowProcW(hwnd: Handle, message: u32, wparam: usize, lparam: isize) -> isize;
        fn DestroyWindow(hwnd: Handle) -> i32;
        fn PostQuitMessage(code: i32);
        fn PostMessageW(hwnd: Handle, message: u32, wparam: usize, lparam: isize) -> i32;
        fn PeekMessageW(msg: *mut Msg, hwnd: Handle, min: u32, max: u32, remove: u32) -> i32;
        fn TranslateMessage(msg: *const Msg) -> i32;
        fn DispatchMessageW(msg: *const Msg) -> isize;
        fn GetClientRect(hwnd: Handle, rect: *mut Rect) -> i32;
        fn GetWindowRect(hwnd: Handle, rect: *mut Rect) -> i32;
        fn SetWindowPos(hwnd: Handle, after: Handle, x: i32, y: i32, width: i32, height: i32, flags: u32) -> i32;
        fn GetAsyncKeyState(key: i32) -> i16;
        fn GetForegroundWindow() -> Handle;
        fn SetWindowTextW(hwnd: Handle, text: *const u16) -> i32;
        fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
        fn GetCursorPos(point: *mut Point) -> i32;
        fn ScreenToClient(hwnd: Handle, point: *mut Point) -> i32;
    }
    #[cfg(not(feature = "offscreen"))]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Handle;
    }
    #[cfg(not(feature = "offscreen"))]
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    #[cfg(not(feature = "offscreen"))]
    unsafe extern "system" fn wndproc(hwnd: Handle, msg: u32, w: usize, l: isize) -> isize {
        if let Some(result) = unsafe { crate::presentation::paint(hwnd, msg) } { return result; }
        match msg {
            0x10 => {
                unsafe { DestroyWindow(hwnd) };
                0
            }
            2 => {
                unsafe { PostQuitMessage(0) };
                0
            }
            0x14 => 1,
            0x20b | 0x20c if crate::input::windows_mouse_button(msg, w).is_some() => 1,
            // AWT consumes typed messages; only system Alt+Space reaches Windows.
            0x102 | 0x106 if crate::windows_keys::character_message(msg, w as u16, l).is_some() => 0,
            _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
        }
    }
    #[cfg(not(feature = "offscreen"))]
    struct Window(Handle);
    #[cfg(not(feature = "offscreen"))]
    impl Drop for Window {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    // AWT delivers focus/close events independently of whether the canvas draws.
    // Keep the same ownership rule while this native loop has no drawable area.
    #[cfg(not(feature = "offscreen"))]
    fn pump_window(window:Handle,mut event:impl FnMut(&Msg)->Result<()>)->Result<bool> {
        let mut msg:Msg=unsafe {std::mem::zeroed()};
        while unsafe {PeekMessageW(&mut msg,0,0,0,1)}!=0 {
            if msg.message==0x12 {return Ok(false);}
            if msg.hwnd==window {event(&msg)?;}
            unsafe {TranslateMessage(&msg);DispatchMessageW(&msg);}
        }
        Ok(true)
    }

    #[cfg(not(feature = "offscreen"))]
    #[test]
    fn zero_area_window_dispatches_restore_focus_and_close() -> Result<()> {
        // Hidden owned windows and this thread's queue only; no desktop focus.
        unsafe extern "system" fn proc(hwnd:Handle,msg:u32,w:usize,l:isize)->isize {
            if msg==0x8001 {
                let _=resize_client_window(hwnd,[32,24]);
                return 0;
            }
            unsafe {wndproc(hwnd,msg,w,l)}
        }
        #[link(name="user32")]
        unsafe extern "system" {fn UnregisterClassW(name:*const u16,instance:Handle)->i32;}
        struct Class(Vec<u16>,Handle);
        impl Drop for Class {fn drop(&mut self){unsafe {UnregisterClassW(self.0.as_ptr(),self.1);}}}
        let instance=unsafe {GetModuleHandleW(ptr::null())};
        let owned_class=Class(wide("OSRSRustZeroAreaRegression"),instance);
        let class=WndClass{size:std::mem::size_of::<WndClass>() as u32,style:0,proc:Some(proc),
            class_extra:0,window_extra:0,instance,icon:0,cursor:0,background:0,
            menu:ptr::null(),name:owned_class.0.as_ptr(),icon_small:0};
        assert_ne!(unsafe {RegisterClassExW(&class)},0);
        let mut window=Window(unsafe {CreateWindowExW(0,owned_class.0.as_ptr(),owned_class.0.as_ptr(),0,
            -10000,-10000,0,0,0,0,instance,ptr::null_mut())});
        assert_ne!(window.0,0);
        let name=wide("STATIC");let other=Window(unsafe {CreateWindowExW(0,name.as_ptr(),name.as_ptr(),0,
            -10000,-10000,0,0,0,0,instance,ptr::null_mut())});assert_ne!(other.0,0);
        // Creation applies desktop minimum tracking sizes; explicitly resize the hidden fixture.
        resize_client_window(window.0,[0,0])?;
        let mut rect:Rect=unsafe {std::mem::zeroed()};
        assert_ne!(unsafe {GetClientRect(window.0,&mut rect)},0);assert_eq!([rect.right,rect.bottom],[0,0]);
        let mut frame=crate::input::Input::default();let mut tick=crate::input::Input::buffered();
        for input in [&mut frame,&mut tick] {input.apply(crate::input::Update::Key(10,true));input.apply(crate::input::Update::Button(0,true));}
        for (hwnd,msg) in [(other.0,8),(window.0,8),(window.0,7),(window.0,0x8001)] {
            assert_ne!(unsafe {PostMessageW(hwnd,msg,0,0)},0);
        }
        assert_ne!(unsafe {PostMessageW(window.0,0x100,65,0)},0);
        let mut ordinary_keys=0;
        let mut focus=Vec::new();
        assert!(pump_window(window.0,|msg| {
            if msg.message==0x100 {ordinary_keys+=1;}
            if !matches!(msg.message,7|8) {return Ok(());}
            let active=msg.message==7;
            focus.push(active);
            if !active {crate::input::dispatch(crate::input::Update::LoseFocus,[&mut frame,&mut tick],None,|_|None,|_|false)?;}
            Ok(())
        })?);
        assert_eq!(focus,[false,true]);
        assert_eq!(ordinary_keys,1);
        for input in [&frame,&tick] {assert!(input.held.is_empty());assert_eq!(input.buttons,0);}
        assert_ne!(unsafe {GetClientRect(window.0,&mut rect)},0);assert_eq!([rect.right,rect.bottom],[32,24]);
        resize_client_window(window.0,[0,0])?;
        assert_ne!(unsafe {PostMessageW(window.0,0x10,0,0)},0);
        assert!(!pump_window(window.0,|_|Ok(()))?,"WM_CLOSE dispatch must reach WM_QUIT at zero area");
        window.0=0;
        Ok(())
    }

    #[cfg(not(feature = "offscreen"))]
    fn resize_client_window(hwnd: Handle, size: [i32; 2]) -> Result<()> {
        let mut outer: Rect = unsafe { std::mem::zeroed() };
        let mut inner: Rect = unsafe { std::mem::zeroed() };
        if unsafe { GetWindowRect(hwnd, &mut outer) } == 0 || unsafe { GetClientRect(hwnd, &mut inner) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { SetWindowPos(hwnd, 0, 0, 0,
            outer.right - outer.left + size[0] - inner.right,
            outer.bottom - outer.top + size[1] - inner.bottom, 0x416) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        // SWP_NOSENDCHANGING bypasses default desktop tracking limits only for this
        // owned client window: https://learn.microsoft.com/windows/win32/winmsg/wm-windowposchanging
        if unsafe { GetClientRect(hwnd, &mut inner) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        println!("Native resize requested {size:?}, actual client [{}, {}]", inner.right, inner.bottom);
        if [inner.right, inner.bottom] != size {
            return Err(Error::Io(format!("native resize request {size:?} constrained to [{}, {}]", inner.right, inner.bottom)));
        }
        Ok(())
    }
    #[cfg(not(feature = "offscreen"))]
    #[test]
    fn native_initial_client_geometry_is_not_silently_desktop_clipped() {
        let class = wide("STATIC");
        let window = Window(unsafe { CreateWindowExW(0, class.as_ptr(), class.as_ptr(),
            0x00cf0000, i32::MIN, i32::MIN, 781, 542, 0, 0, 0, ptr::null_mut()) });
        assert_ne!(window.0, 0);
        resize_client_window(window.0, [765, 503]).unwrap();
        let mut rect: Rect = unsafe { std::mem::zeroed() };
        assert_ne!(unsafe { GetClientRect(window.0, &mut rect) }, 0);
        assert_eq!([rect.right, rect.bottom], [765, 503]);
        resize_client_window(window.0, [960, 640]).unwrap();
        assert_ne!(unsafe { GetClientRect(window.0, &mut rect) }, 0);
        assert_eq!([rect.right, rect.bottom], [960, 640]);
    }
    #[cfg(not(feature = "offscreen"))]
    pub fn interface_view(cache_path: &str, group: u16, smoke: bool) -> Result<()> {
        interface_loop(cache_path, Some(group), None, false, smoke, false, false)
    }
    #[cfg(not(feature = "offscreen"))]
    pub fn live_view(
        cache_path: &str,
        address: &str,
        username: &str,
        password_file: &str,
        key: &str,
        smoke: bool,
        exercise: bool,
        inventory: bool,
    ) -> Result<()> {
        interface_loop(
            cache_path,
            None,
            Some((address, username, password_file, key)),
            false,
            smoke,
            exercise,
            inventory,
        )
    }
    #[cfg(not(feature = "offscreen"))]
    pub fn live_token_view(cache_path: &str,address: &str,username: &str,config_file: &str,key: &str,
        smoke: bool,exercise: bool,inventory: bool) -> Result<()> {
        interface_loop(cache_path,None,Some((address,username,config_file,key)),true,smoke,exercise,inventory)
    }
    #[cfg(not(feature = "offscreen"))]
    #[cfg(feature = "custom-assets")]
    pub fn live_view_with_cache(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,smoke:bool)->Result<()> {
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,smoke,false,false,Some(cache), #[cfg(feature="private-test-adapter")] None, #[cfg(feature="native-profiling")] None, #[cfg(feature="enhanced-renderer")] None)
    }
    #[cfg(feature="private-test-adapter")]
    pub fn private_authored_test(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,directory:std::path::PathBuf,headless:bool)->Result<()> {
        #[cfg(feature = "offscreen")]
        if !headless { return Err(Error::Invalid("offscreen build requires --headless")); }
        let mut adapter=crate::private_adapter::Adapter::new(directory)?;adapter.headless=headless;
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,false,false,false,Some(cache),Some(adapter), #[cfg(feature="native-profiling")] None, #[cfg(feature="enhanced-renderer")] None)
    }
    #[cfg(all(feature="native-profiling",feature="custom-assets"))]
    pub fn profiled_authored_live(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,directory:std::path::PathBuf,smoke:bool)->Result<()> {
        std::fs::create_dir_all(&directory)?;
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,smoke,false,false,Some(cache),#[cfg(feature="private-test-adapter")] None,Some((directory,crate::ui_profile::Benchmark::default())), #[cfg(feature="enhanced-renderer")] None)
    }
    #[cfg(all(feature="native-profiling",feature="enhanced-renderer",feature="custom-assets"))]
    pub fn profiled_enhanced_authored_live(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,directory:std::path::PathBuf,settings:crate::enhanced::Settings,benchmark:crate::ui_profile::Benchmark,smoke:bool)->Result<()> {
        settings.validate().map_err(Error::Io)?;
        std::fs::create_dir_all(&directory)?;
        eprintln!("Explicit profiling benchmark: {benchmark:?}");
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,smoke,false,false,Some(cache),#[cfg(feature="private-test-adapter")] None,Some((directory,benchmark)),Some(settings))
    }
    #[cfg(all(feature="enhanced-renderer",feature="custom-assets"))]
    pub fn enhanced_authored_live(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,settings:crate::enhanced::Settings,smoke:bool)->Result<()> {
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,smoke,false,false,Some(cache),#[cfg(feature="private-test-adapter")] None,#[cfg(feature="native-profiling")] None,Some(settings))
    }
    #[cfg(all(feature="enhanced-renderer",feature="private-test-adapter"))]
    pub fn enhanced_private_authored_test(cache:DiskCache,address:&str,username:&str,password_file:&str,key:&str,directory:std::path::PathBuf,settings:crate::enhanced::Settings,headless:bool)->Result<()> {
        let mut adapter=crate::private_adapter::Adapter::new(directory)?;adapter.headless=headless;
        interface_loop_with_cache("",None,Some((address,username,password_file,key)),false,false,false,false,Some(cache),Some(adapter),#[cfg(feature="native-profiling")] None,Some(settings))
    }
    #[cfg(not(feature = "offscreen"))]
    fn interface_loop(cache_path:&str,group:Option<u16>,login:Option<(&str,&str,&str,&str)>,token_login:bool,smoke:bool,exercise:bool,inventory:bool)->Result<()> {
        interface_loop_with_cache(cache_path,group,login,token_login,smoke,exercise,inventory,None, #[cfg(feature="private-test-adapter")] None, #[cfg(feature="native-profiling")] None, #[cfg(feature="enhanced-renderer")] None)
    }
    fn interface_loop_with_cache(
        cache_path: &str,
        group: Option<u16>,
        login: Option<(&str, &str, &str, &str)>,
        token_login: bool,
        smoke: bool,
        exercise: bool,
        inventory: bool,
        provided_cache: Option<DiskCache>,
        #[cfg(feature="private-test-adapter")] mut adapter:Option<crate::private_adapter::Adapter>,
        #[cfg(feature="native-profiling")] profile_directory:Option<(std::path::PathBuf,crate::ui_profile::Benchmark)>,
        #[cfg(feature="enhanced-renderer")] enhanced_settings:Option<crate::enhanced::Settings>,
    ) -> Result<()> {
        use crate::{
            client_state::ClientState, interface_packet::Update, interface_render::Renderer,
            raster::Raster,
        };
        let mut cache = if let Some(cache)=provided_cache {cache} else if let Some((address, _, _, _)) = login {
            DiskCache::open_remote(cache_path, 64 << 20, address.parse().map_err(|_| Error::Invalid("server address"))?)?
        } else { DiskCache::open(cache_path, 64 << 20)? };
        if login.is_some() {
            cache.wait_for_recovery(true);
            // GameIntProjection.ab/co.ac loads the Huffman wordpack before interfaces.
            crate::huffman::Huffman::new(&cache.named_file(10, "huffman", "")?)?;
            // GameIntProjection.ab/co.az completes interfaces, scripts and fonts
            // before login, so one-shot listener roots are available when invoked.
            for archive in [3, 23, 12, 13] {
                if archive == 23 && !cache.has_archive(23)? { continue; }
                if !cache.archive_ready(archive)? {
                    return Err(Error::Io(format!("startup archive {archive} is still pending")));
                }
            }
            println!("Native startup interface/script/font archives ready: {:?}", cache.recovery_stats());
        }
        let mut client = ClientState::load(cache)?;
        #[cfg(feature="native-profiling")]
        if let Some((dir,_))=&profile_directory {client.ui_profile=Some(crate::ui_profile::Profile::new(dir.join("ui-profile.csv")));}
        let mut profile = if let Some((address, _, _, key)) = login {
            let public_key = std::fs::read(key)?;
            match crate::saved_preferences::Profile::open(
                std::path::Path::new("client-data/profiles"),
                address,
                &public_key,
            ) {
                Ok((profile, bytes)) => {
                    if let Some(bytes) = bytes {
                        if let Err(error) = client.preferences.load_saved(&bytes) {
                            eprintln!("Could not decode native preferences: {error}");
                        }
                    }
                    Some(profile)
                }
                Err(error) => {
                    eprintln!("Could not load native preferences: {error}");
                    None
                }
            }
        } else {
            None
        };
        let mut varc_profile = if let Some((address, _, _, key)) = login {
            let public_key = std::fs::read(key)?;
            match crate::saved_preferences::Profile::open(
                std::path::Path::new("client-data/varcs"),
                address,
                &public_key,
            ) {
                Ok((profile, bytes)) => {
                    if let Some(bytes) = bytes {
                        if let Err(error) = client.scripts.variables.load_varcs(&bytes) {
                            eprintln!("Could not fully decode native client variables: {error}");
                        }
                    }
                    Some(profile)
                }
                Err(error) => {
                    eprintln!("Could not load native client variables: {error}");
                    None
                }
            }
        } else {
            None
        };
        let resize_probe = smoke && exercise && std::env::var_os("OSRS_SMOKE_RESIZE").is_some();
        if login.is_some() { client.set_window_mode(if resize_probe { 2 } else { client.preferences.preferred_window_mode }); }
        if let Some(group) = group {
            client.interface(Update::OpenTop(group))?;
        }
        if let Ok(value) = std::env::var("OSRS_REQUIRED_EULA") {
            client.preferences.required_eula=value.parse().map_err(|_| crate::Error::Invalid("required EULA revision"))?;
        }
        let mut live = login
            .map(|(address, username, password, key)| {
                if token_login {
                    let config=crate::token_auth::Config::from_json(&std::fs::read_to_string(password)?)?;
                    crate::live_runtime::Connection::connect_token(&mut client,address,username,config,key)
                } else {
                    crate::live_runtime::Connection::connect(&mut client,address,username,password,key)
                }
            })
            .transpose()?;
        // #181: the journal starts at the first main-loop poll, so connect/login/EULA/title polls are recorded calls.
        #[cfg(feature="private-test-adapter")]
        if let (Some(adapter),Some(live))=(adapter.as_mut(),live.as_mut()) { adapter.attach_journal(live)?; }
        #[cfg(feature="enhanced-renderer")]
        if let (Some(settings),Some(live))=(enhanced_settings,live.as_mut()) {live.set_enhanced(settings)?;}
        #[cfg(feature="native-profiling")]
        if let (Some((dir,benchmark)),Some(live))=(&profile_directory,live.as_mut()) {
            live.scene_profile=Some(crate::ui_profile::Profile::new(dir.join("scene-profile.csv")));
            live.scene_filter=benchmark.scene_filter();
        }
        #[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
        if let (Some((dir,_)),Some(live))=(&profile_directory,live.as_mut()) {
            match crate::enhanced_trace::Trace::new(&dir.join("enhanced-trace.jsonl")) {
                Ok(trace)=>live.enhanced_trace=Some(trace),
                Err(error)=>eprintln!("Enhanced trace unavailable: {error}"),
            }
        }
        while let Some(event) = client.next_interface_event()? {
            client.run_event(event)?;
        }
        let mut renderer = Renderer::load(
            &mut client.scripts.source.cache,
            client.preferences.device.brightness,
        )?;
        if login.is_some() {
            // The original startup sprite stage admits the authentic crosses
            // before play, rather than letting the first click expire loading.
            renderer.click_cross_sprites.load(&mut client.scripts.source.cache)?;
        }
        #[cfg(not(feature = "offscreen"))]
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        #[cfg(not(feature = "offscreen"))]
        let name = wide("OSRSRustInterface");
        #[cfg(not(feature = "offscreen"))]
        let class = WndClass {
            size: std::mem::size_of::<WndClass>() as u32,
            style: 3,
            proc: Some(wndproc),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: 0,
            cursor: unsafe { LoadCursorW(0, 32512usize as *const u16) },
            background: 0,
            menu: ptr::null(),
            name: name.as_ptr(),
            icon_small: 0,
        };
        #[cfg(not(feature = "offscreen"))]
        if unsafe { RegisterClassExW(&class) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        #[cfg(not(feature = "offscreen"))]
        let title = wide(&format!("OSRS 239 Rust â€” interface {group:?}"));
        #[cfg(feature="native-profiling")]
        let hidden=smoke || profile_directory.as_ref().is_some_and(|(_,b)|b.headless());
        #[cfg(not(feature = "offscreen"))]
        #[cfg(not(feature="native-profiling"))]
        let hidden=smoke;
        #[cfg(not(feature = "offscreen"))]
        #[cfg(feature="private-test-adapter")]
        let hidden=hidden || adapter.as_ref().is_some_and(|adapter|adapter.headless);
        #[cfg(not(feature = "offscreen"))]
        let window = Window(unsafe {
            CreateWindowExW(
                0,
                name.as_ptr(),
                title.as_ptr(),
                if hidden { 0x00cf0000 } else { 0x10cf0000 },
                i32::MIN,
                i32::MIN,
                781,
                542,
                0,
                0,
                instance,
                ptr::null_mut(),
            )
        });
        #[cfg(not(feature = "offscreen"))]
        if window.0 == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        #[cfg(not(feature = "offscreen"))]
        // CreateWindowExW may silently constrain the requested outer size to a
        // narrow desktop. Establish the intended initial client area before the
        // original engine computes its canvas and aligns fixed root widgets.
        resize_client_window(window.0, [765, 503])?;
        #[cfg(feature="native-profiling")]
        if let Some((_,benchmark))=&profile_directory {
            if let Some(zoom)=benchmark.zoom(){client.viewport.zoom=zoom;}
            if let Some(size)=benchmark.window(){client.set_window_mode(2);resize_client_window(window.0,size)?;}
        }
        #[cfg(not(feature = "offscreen"))]
        let mut presenter = unsafe { crate::presentation::Presenter::new(window.0) }?;
        #[cfg(feature="native-profiling")]
        let mut presentation_trace=profile_directory.as_ref().and_then(|(dir,_)| {
            match crate::presentation_trace::Trace::new(&dir.join("presentation-trace.csv")) {
                Ok(trace)=>{presenter.track_events();Some(trace)},
                Err(error)=>{eprintln!("Presentation trace disabled: {error}");None},
            }
        });
        #[cfg(feature="native-profiling")]
        let mut zero_area_probe=profile_directory.as_ref().and_then(|(_,b)|b.zero_area_frames())
            .map(|n|(n,None::<([i32;2],i32,i32)>));
        #[cfg(feature="native-profiling")]
        let mut benchmark_run=profile_directory.as_ref().and_then(|(_,b)|b.run());
        #[cfg(feature="native-profiling")]
        let mut benchmark_rows=Vec::<String>::new();
        // Benchmark captures are PNG-only. The warm-up frame is copied here (last Warmed wins after a
        // readiness reset) and encoded only at Complete, so no encode or write runs inside the sampled
        // window. This moves work out of the window: runs before and after this change are not a matched
        // performance pair.
        #[cfg(feature="native-profiling")]
        let mut benchmark_start=None::<Raster>;
        let mut presentation_errors = 0u64;
        let mut resize_probe_phase = 0;
        #[cfg(not(feature = "offscreen"))]
        if resize_probe { resize_client_window(window.0, [1024, 768])?; }
        if let Some(live) = live.as_mut() { live.start_mouse_recorder()?; }
        let mut mouse_tracking = false;
        let mut actions = crate::live_smoke::Actions::new(inventory);
        let mut extra_smoke_clicks = 0usize;
        let mut editor_probe=crate::native_editor_probe::Probe::default();
        let mut pending_clicks = std::collections::VecDeque::new();
        let mut frames = 0;
        let audio_worker = crate::audio_worker::Worker::start(client.audio.mixer())?;
        let simulation_epoch=Instant::now();
        let mut game_clock=crate::game_clock::Clock::default();
        let mut tick_input=crate::input::Input::buffered();
        let mut mouse_wheel = crate::input::WheelAccumulator::default();
        let render_epoch = Instant::now();
        client.scripts.source.cache.wait_for_recovery(false);
        'main: loop {
            let start = Instant::now();
            #[cfg(feature="native-profiling")]
            if let Some((_,benchmark))=&profile_directory {
                if let Some(run)=&benchmark_run {
                    client.camera.pitch=2048;
                    client.camera.yaw=benchmark.sample_yaw(run.sample_index());
                } else {client.camera.yaw=benchmark.yaw(client.camera.yaw);}
            }
            if client.scripts.variables.varcs_due(render_epoch.elapsed()) {
                if let Some(saved) = varc_profile.as_mut() {
                    if let Err(error) = client
                        .scripts
                        .variables
                        .encode_varcs()
                        .and_then(|bytes| saved.save(&bytes))
                    {
                        eprintln!("Could not save native client variables: {error}");
                    }
                    client.scripts.variables.varcs_saved(render_epoch.elapsed());
                }
            }
            if let Some(saved) = profile.as_mut() {
                if let Err(error) = client
                    .preferences
                    .encode_saved()
                    .and_then(|bytes| saved.save(&bytes))
                {
                    eprintln!("Could not save native preferences: {error}");
                    profile = None;
                }
            }
            #[cfg(feature="native-profiling")]
            if let Some((remaining,saved))=zero_area_probe.as_mut() {
                if let Some((size,cycle,server_tick))=*saved {
                    if *remaining==1 {
                        resize_client_window(window.0,size)?;
                        println!("Native zero-area probe restored: cycle {cycle}->{}; server_tick {server_tick}->{}",client.cycle,client.server_tick);
                        if client.cycle==cycle {return Err(Error::Invalid("zero-area simulation did not advance"));}
                        zero_area_probe=None;
                    } else {*remaining-=1;}
                } else if live.as_ref().is_some_and(|live|live.observation_ready(&client)) {
                    let mut rect:Rect=unsafe {std::mem::zeroed()};
                    if unsafe {GetClientRect(window.0,&mut rect)}==0 {return Err(std::io::Error::last_os_error().into());}
                    if rect.right>0 && rect.bottom>0 {
                        *saved=Some(([rect.right,rect.bottom],client.cycle,client.server_tick));
                        resize_client_window(window.0,[0,0])?;
                        println!("Native zero-area probe started: frames {}; cycle {}; server_tick {}",remaining,client.cycle,client.server_tick);
                    }
                }
            }
            #[cfg(not(feature = "offscreen"))]
            let mut rect: Rect = unsafe { std::mem::zeroed() };
            #[cfg(not(feature = "offscreen"))]
            unsafe {
                GetClientRect(window.0, &mut rect);
            }
            #[cfg(feature = "offscreen")]
            // #233 offscreen: no window; the client area is the fixed 765x503 logical canvas.
            let rect = Rect { left: 0, top: 0, right: 765, bottom: 503 };
            let drawable=rect.right>0 && rect.bottom>0;
            let physical = [rect.right, rect.bottom];
            // A minimized native surface does not resize the logical game canvas
            // or suspend the engine. Restore resumes ordinary layout and presentation.
            let fit = if !drawable {
                crate::canvas_transform::Fit::new(client.canvas,client.canvas)
            } else if let Some(connection) = &live {
                let game_state = if connection.title.is_some() { 10 }
                    else if connection.session.is_established() { if connection.world.is_some() { 30 } else { 25 } }
                    else { 0 };
                client.native_layout(physical, render_epoch.elapsed().as_millis() as u64, game_state)?
            } else {
                client.resize_canvas(physical)?;
                crate::canvas_transform::Fit::new(client.canvas, physical)
            };
            client.input.begin_cycle();
            let clock_step=game_clock.advance(simulation_epoch.elapsed().as_nanos().min(i64::MAX as u128) as i64);
            client.cycle=client.cycle.wrapping_add(clock_step.ticks as i32);
            #[cfg(not(feature = "offscreen"))]
            if resize_probe && resize_probe_phase == 0 && live.as_ref().is_some_and(|v| v.world.is_some()) {
                println!("Native resize acceptance initial mode {}, canvas {:?}, physical {:?}", client.preferences.window_mode, client.canvas, physical);
                if client.canvas != [1024, 768] { return Err(Error::Invalid("native initial logical resize")); }
                let mut stack = crate::vm::Stacks::default(); stack.push_int(1)?;
                crate::vm::Host::extended(&mut client, 5307, false, &mut stack)?;
                resize_probe_phase = 1;
            } else if resize_probe && resize_probe_phase == 1 && client.canvas == [765, 503] && client.window.deadline.is_none() {
                println!("Native resize acceptance fixed canvas {:?}, origin {:?}", client.canvas, fit.origin);
                if fit.origin != [129, 0] { return Err(Error::Invalid("native fixed canvas origin")); }
                let mut stack = crate::vm::Stacks::default(); stack.push_int(2)?;
                crate::vm::Host::extended(&mut client, 5307, false, &mut stack)?;
                resize_client_window(window.0, [960, 640])?;
                resize_probe_phase = 2;
            } else if resize_probe && resize_probe_phase == 2 && client.canvas == [960, 640] && client.window.deadline.is_none() {
                println!("Native resize acceptance final canvas {:?}, origin {:?}; delayed status queued", client.canvas, fit.origin);
                resize_probe_phase = 3;
            }
            #[cfg(not(feature = "offscreen"))]
            if exercise && (!resize_probe || resize_probe_phase == 3) {
                if let Some(live) = live.as_ref() {
                    for (key, down) in actions.next_keys(&client, live) {
                        if unsafe {
                            PostMessageW(
                                window.0,
                                if down { 0x100 } else { 0x101 },
                                key as usize,
                                0,
                            )
                        } == 0
                        {
                            return Err(std::io::Error::last_os_error().into());
                        }
                    }
                    if let Some(point) = actions.next_click(&mut client, live, fit)? {
                        let right = std::mem::take(&mut actions.right_click);
                        let [x, y] = fit.to_physical(point);
                        let point = ((y as isize) << 16) | (x as isize & 65535);
                        let drag = std::mem::take(&mut actions.drag_input);
                        let messages: &[(u32, usize)] = match drag {
                            1 => &[(0x200, 0), (0x201, 1)],
                            2 => &[(0x200, 1)],
                            3 => &[(0x200, 1), (0x202, 0)],
                            _ if right => &[(0x200, 0), (0x204, 2), (0x205, 0)],
                            _ => &[(0x200, 0), (0x201, 1), (0x202, 0)],
                        };
                        let extra = if drag == 0 && !right && std::env::var_os("OSRS_SMOKE_EXTRA_BUTTONS").is_some() {
                            extra_smoke_clicks += 1;
                            let button = 1 + (extra_smoke_clicks - 1) % 2;
                            eprintln!("Native extra mouse acceptance: XBUTTON{button}");
                            button
                        } else { 0 };
                        for &(message, button) in messages {
                            let (message, button) = if extra != 0 && matches!(message, 0x201 | 0x202) {
                                let down = message == 0x201;
                                (if down { 0x20b } else { 0x20c }, (extra << 16) | if down { 32 << (extra - 1) } else { 0 })
                            } else { (message, button) };
                            if unsafe { PostMessageW(window.0, message, button, point) } == 0 {
                                return Err(std::io::Error::last_os_error().into());
                            }
                        }
                    }
                }
            }
            #[cfg(not(feature = "offscreen"))]
            if exercise && inventory && actions.complete {
                for (message,wparam,lparam) in editor_probe.next(&mut client,fit)? {
                    if unsafe {PostMessageW(window.0,message,wparam,lparam)}==0 {return Err(std::io::Error::last_os_error().into());}
                }
            }
            #[cfg(not(feature = "offscreen"))]
            if !pump_window(window.0,|msg| {
                    use crate::input::Update;
                    let route_editor=!live.as_ref().is_some_and(|v|v.is_reconnecting() || v.logged_out().is_some() || v.title.is_some());
                    let mut apply = |update| {
                        crate::input::dispatch(update,[&mut client.input,&mut tick_input],route_editor.then_some(&mut client.interfaces),crate::clipboard::read,crate::clipboard::write)
                    };
                    match msg.message {
                        0x200 => {
                            let point = fit.to_logical([msg.lparam as i16 as i32, (msg.lparam >> 16) as i16 as i32]);
                            apply(Update::Pointer(point))?;
                            if let Some(live) = live.as_ref() { live.mouse_pointer(point); }
                            // Hidden smoke windows receive synthetic movement rather than the OS cursor.
                            if !smoke && !mouse_tracking {
                                let mut track = TrackMouse { size: std::mem::size_of::<TrackMouse>() as u32, flags: 2, window: window.0, hover_millis: 0 };
                                mouse_tracking = unsafe { TrackMouseEvent(&mut track) } != 0;
                            }
                        }
                        0x2a3 => {
                            mouse_tracking = false;
                            apply(Update::Pointer([-1, -1]))?;
                            if let Some(live) = live.as_ref() { live.mouse_pointer([-1, -1]); }
                        }
                        0x201 | 0x202 | 0x204 | 0x205 | 0x207 | 0x208 | 0x20b | 0x20c => {
                            if let Some((physical, down)) = crate::input::windows_mouse_button(msg.message, msg.wparam) {
                                let point = fit.to_logical([
                                    msg.lparam as i16 as i32,
                                    (msg.lparam >> 16) as i16 as i32,
                                ]);
                                let button = mouse_button(physical);
                                apply(Update::ButtonAt(button, down, point))?;
                                if down { if let Some(live) = live.as_mut() { live.mouse_click([1, 2, 4][button as usize], client.input.press_position); } }
                                mouse_capture(window.0, down, msg.wparam);
                            }
                        }
                        0x20a | 0x20e => {
                            let delta = mouse_wheel.rotate((msg.wparam >> 16) as i16, msg.message == 0x20e);
                            apply(Update::Wheel(delta))?;
                            if let Some(live) = live.as_mut() { live.mouse_wheel(delta); }
                        },
                        0x100 | 0x101 | 0x104 | 0x105 => {
                            let code = crate::windows_keys::KEYS.get(msg.wparam).copied().unwrap_or(-1);
                            let down = msg.message == 0x100 || msg.message == 0x104;
                            if code >= 0 && (!down || code & 128 == 0) {
                                apply(Update::Key(code & 127, down))?;
                                if let Some(live) = live.as_mut() { live.keyboard_key(code & 127, down); }
                            }
                            if down {
                                if let Some(ch) = crate::windows_keys::down_character(msg.wparam) {
                                    apply(Update::Character(ch))?;
                                }
                            }
                        }
                        0x102 | 0x106 => {
                            if let Some(ch) = crate::windows_keys::character_message(msg.message, msg.wparam as u16, msg.lparam) {
                                apply(Update::Character(ch))?;
                            }
                        },
                        7 => { if let Some(live) = live.as_mut() { live.window_focus(true); } },
                        8 => {
                            apply(Update::LoseFocus)?;
                            if let Some(live) = live.as_mut() { live.window_focus(false); }
                        },
                        _ => {}
                    }
                Ok(())
            })? {break 'main;}
            let title_input_frame = live.as_ref().is_some_and(|v| {
                v.is_reconnecting() || v.logged_out().is_some() || v.title.is_some()
            });
            if let Some(live) = live.as_mut() {
                live.poll_elapsed(&mut client, clock_step.ticks)?;
            }
            audio_worker.set_mixer(client.audio.mixer());
            let reconnecting = live.as_ref().is_some_and(|v| {
                v.is_reconnecting() || v.logged_out().is_some() || v.title.is_some()
            });
            if title_input_frame || reconnecting {
                // H-reset: release an adapter close press before the cycle is cleared.
                #[cfg(feature="private-test-adapter")]
                if let Some(adapter)=adapter.as_mut() { adapter.abort_input(&mut tick_input); }
                tick_input.begin_cycle();
                tick_input.discard_key_events();
            }
            let after_network = Instant::now();
            if !reconnecting && !title_input_frame {
                for _ in 0..clock_step.ticks { client.click_cross.tick(); }
            }
            let scripts_busy = live.as_ref().is_some_and(|v| v.scripts_busy())
                || !client.interfaces.scripts.is_empty()
                || clock_step.ticks == 0;
            if !scripts_busy && !reconnecting && !title_input_frame && client.menu.is_none() && client.widget_drag.is_none() {
                let point = if tick_input.pressed_buttons != 0 { tick_input.press_position } else { tick_input.pointer };
                let entries = if let Some(live) = live.as_mut() {
                    live.context_menu(&mut client, point)?
                } else { client.interface_menu_at(point, tick_input.pointer)? };
                client.pointer_menu = entries;
            }
            if let Err(e) = if scripts_busy || reconnecting || title_input_frame {
                Ok(0)
            } else {
                // H-pre: an adapter close press/release is validated and delivered only
                // on a frame whose interface ticks run; each internal tick is observed
                // in the batch callback, and H-post cleans up an unobserved delivery.
                #[cfg(feature="private-test-adapter")]
                if let Some(adapter)=adapter.as_mut() { adapter.before_tick(&mut client, &mut tick_input, clock_step.ticks); }
                let ticked = client.tick_interfaces_elapsed_with(clock_step.ticks, &mut tick_input, false, |client| {
                    if let Some((point, entries)) = client.tick_widget_drag()? {
                        let text_icons = renderer.text_icons.clone();
                        let font = renderer.menu_font(&mut client.scripts.source.cache)?;
                        client.perform_default_menu_with_icons(point, entries, font, text_icons)?;
                    }
                    #[cfg(feature="private-test-adapter")]
                    if let Some(adapter)=adapter.as_mut() { adapter.observe_tick(client); }
                    Ok(())
                });
                #[cfg(feature="private-test-adapter")]
                if let Some(adapter)=adapter.as_mut() { adapter.after_frame(&mut tick_input); }
                ticked
            } {
                if let Some(live) = live.as_mut() {
                    live.report(e.to_string());
                } else {
                    return Err(e);
                }
            }
            while let Some(error) = client.script_failures.pop_front() {
                if let Some(live) = live.as_mut() {
                    live.report(error.to_string());
                } else {
                    return Err(error);
                }
            }
            #[cfg(feature="private-test-adapter")]
            if let (Some(adapter),Some(live))=(adapter.as_mut(),live.as_mut()) {
                let ready=live.private_action_ready(&client) && !reconnecting && !title_input_frame && clock_step.ticks>0;
                let old_pointer=client.input.pointer;
                adapter.poll(&mut client,live,ready)?;
                // Runs on every frame: a delivered close press cannot stay held when ticks stall.
                adapter.release_watchdog(&mut tick_input);
                if client.input.pointer!=old_pointer {tick_input.apply(crate::input::Update::Pointer(client.input.pointer));}
                // Retain the admitted adapter click through the next simulation
                // snapshot, where draw-selected walking dispatches its cross.
                tick_input.press_position=client.input.press_position;
                adapter.inspect_scene(live)?;
            }
            let after_scripts = Instant::now();
            if reconnecting {
                pending_clicks.clear();
                client.menu = None;
                client.widget_drag = None;
                client.pointer_menu.clear();
                client.interfaces.drag_visual = None;
            }
            let menu_buttons = if client.preferences.mouse_camera {
                3
            } else {
                7
            };
            if client.widget_drag.is_some() {
                pending_clicks.clear();
            }
            if !reconnecting
                && client.widget_drag.is_none()
                && client.input.pressed_buttons & menu_buttons != 0
            {
                pending_clicks.push_back((
                    client.input.press_position,
                    client.input.pressed_buttons & 2 != 0,
                ));
            }
            if let Some((point, right)) = if scripts_busy {
                None
            } else {
                pending_clicks.pop_front()
            } {
                let outbound_before = client.outbound.len();
                if right && client.menu.is_some() {
                    // An open menu retains its entries; hover handles leaving its bounds.
                } else if right {
                    let mut entries = vec![crate::context_menu::Entry {
                        option: "Cancel".into(),
                        target: String::new(),
                        action: crate::context_menu::Action::Cancel,
                        submenu: false,
                        children: Vec::new(),
                    }];
                    entries.extend(if let Some(live) = live.as_mut() {
                        live.context_menu(&mut client, point)?
                    } else { client.interface_menu_at(point, client.input.pointer)? });
                    let text_icons = renderer.text_icons.clone();
                        let font = renderer.menu_font(&mut client.scripts.source.cache)?;
                    client.prepare_scene_menu(point);
                    client.menu = Some(crate::context_menu::Menu::open_with_icons(
                        entries,
                        point,
                        client.canvas,
                        font,
                        text_icons,
                    ));
                } else if let Some(menu) = client.menu.take() {
                    if let Some(action) = menu.selected_action(point) {
                        client.menu_action_from_open_menu(*action)?;
                    }
                } else {
                    let entries = if let Some(live) = live.as_mut() {
                        live.context_menu(&mut client, point)?
                    } else { client.interface_menu_at(point, client.input.pointer)? };
                    if client.begin_widget_drag(point, &entries)? {
                        // Deferred until release or a qualified drag completes.
                    } else if !entries.is_empty() {
                        let text_icons = renderer.text_icons.clone();
                        let font = renderer.menu_font(&mut client.scripts.source.cache)?;
                        client.perform_default_menu_with_icons(point, entries, font, text_icons)?;
                    } else if let Some(live) = live.as_mut() {
                        live.click(&mut client, point)?;
                        client.clear_spell()?;
                    }
                }
                if exercise {
                    println!(
                        "Native acceptance dispatch at {point:?}: {:?}",
                        client
                            .outbound
                            .iter()
                            .skip(outbound_before)
                            .map(|p| (p.opcode, p.payload.as_slice()))
                            .collect::<Vec<_>>()
                    );
                }
            }
            if let Some(menu) = &mut client.menu {
                if !menu.hover(
                    client.input.pointer,
                    client.actor_cycle as i64,
                    client.canvas,
                    renderer.menu_font(&mut client.scripts.source.cache)?,
                ) {
                    client.menu = None;
                } else if client.input.wheel != 0 {
                    menu.scroll_wheel(client.input.pointer, client.input.wheel, client.canvas[1]);
                }
            }
            for _ in 0..clock_step.ticks {
                if reconnecting {
                    continue;
                }
                client.tick_actor_sequences()?;
                if let Some(packet) = client.camera.input_report(&client.input.held) {
                    client.outbound.push_back(packet);
                }
                if let Some((scene, _)) = live
                    .as_ref()
                    .and_then(|connection| connection.world.as_ref())
                {
                    if client.camera.mode == 0 {
                        if let Some((target, plane, height)) = live.as_ref().unwrap().camera_follow_target(&client) {
                            client.camera.tick_follow(
                                scene,
                                target,
                                plane,
                                client.preferences.camera_follow_height,
                            );
                            client.camera.set_target_height(height, client.preferences.camera_follow_height);
                        }
                    } else if client.camera.mode == 1 {
                        if let Some((target, _, _)) = live.as_ref().unwrap().camera_follow_target(&client) {
                            client.camera.recenter_free(scene, target, client.data.position[0] as usize, client.preferences.camera_follow_height);
                        }
                        if client.camera.tick_free(&client.input.held, client.preferences.camera_speed) {
                            client.outbound.push_back(crate::protocol::Packet {
                                opcode: 5,
                                payload: Vec::new(),
                            });
                        }
                    }
                    client.camera.tick_input(
                        &client.input.held,
                        client.input.pointer,
                        client.preferences.mouse_camera && client.input.buttons & 4 != 0,
                    );
                    client
                        .camera
                        .tick_locked(scene, client.data.position[0] as usize);
                    for shake in &mut client.camera.shake {
                        shake.elapsed = shake.elapsed.wrapping_add(1);
                    }

                }
            }
            let fraction = clock_step.interpolation;
            for world in client.world_entities.values_mut() {
                for entity in world.entities.values_mut() {
                    entity.advance(client.actor_cycle, fraction, 33);
                }
            }
            client.audio.set_listener(client.local_minimap_fine());
            let sound_worlds=crate::world_audio::Context {
                root: client.world_entities.get(&0),
                base: client.world_base,
                local_view: client.player_state.as_ref().map_or(0,|s|s.local_view),
            };
            for _ in 0..clock_step.ticks {
                client.audio.tick_in_worlds(&mut client.scripts.source.cache, &client.preferences.device, Some(&sound_worlds))?;
            }
            let layout = client.interfaces.layout(client.canvas)?;
            let mut raster = Raster::new(
                client.canvas[0] as usize,
                client.canvas[1] as usize,
                16 << 20,
            )?;
            raster.clear(0xff29251f);
            if let Some(live) = live.as_mut() {
                raster = live.paint(
                    &mut client,
                    fit.logical[0] as usize,
                    fit.logical[1] as usize,
                )?;
            }
            renderer.set_brightness(&mut client.scripts.source.cache,client.preferences.device.brightness)?;
            renderer.minimap = live.as_ref().and_then(|live| live.minimap_view(&client));
            renderer.world_drawn = live.as_ref().is_some_and(|live| live.world.is_some());
            renderer.map_overview = if layout.widgets.iter().any(|(id, placed)| !placed.hidden && client.interfaces.widgets.get(id).is_some_and(|w| w.content == 1401)) {
                client.world_map.overview_image(&mut client.scripts.source.cache)?
            } else { None };
            renderer.map_detail.clear();
            for id in &layout.order {
                let Some(placed) = layout.widgets.get(id).filter(|p| !p.hidden && p.geometry.size.iter().all(|&n| n > 0)) else { continue; };
                if client.interfaces.widgets.get(id).is_some_and(|w| w.content == 1400) {
                    client.world_map.render_context(client.cycle, &client.scripts.variables);
                    if let Some(image) = client.world_map.detailed_image(
                        &mut client.scripts.source.cache, client.data.members,
                        placed.geometry.size, client.preferences.device.brightness,
                    )? { client.world_map.display_origin=placed.absolute; let image=client.world_map.compose_heatmap(image,&mut client.heatmap)?; renderer.map_detail.insert(*id, image); }
                }
            }

            renderer.map_overview_flash=if layout.widgets.iter().any(|(id,p)|!p.hidden&&client.interfaces.widgets.get(id).is_some_and(|w|w.content==1401)) {
                client.world_map.overview_markers(&mut client.scripts.source.cache,client.data.members)?
            } else {None};
            renderer.metrics =
                client
                    .preferences
                    .device
                    .display_fps
                    .then_some(crate::client_metrics::Display {
                        fps: client.frame_rate.fps,
                        memory_kib: crate::client_metrics::heap_kib(),
                        low_detail: false,
                    });
            renderer.preview = if layout.widgets.iter().any(|(id, placed)| !placed.hidden && client.interfaces.widgets.get(id).is_some_and(|w| w.content == 328)) { live.as_ref().and_then(|live| live.player_preview(&client)) } else { None };
            renderer.designer = client.appearance_editor.clone();
            renderer.cycle = client.cycle;
            renderer.player_pose = client.player_state.as_ref().and_then(|s| s.local_actor())
                .and_then(|p| p.appearance.as_ref().map(|a| crate::widget_appearance::pose(a.animations[0], &p.pose.state)));
            renderer.player = client
                .player_state
                .as_ref()
                .and_then(|s| s.local_actor())
                .and_then(|p| p.appearance.as_ref())
                .map(|a| a.appearance.clone());
            renderer.active_npcs = client
                .npc_state
                .npcs
                .iter()
                .map(|(&index, npc)| (index, (npc.id, npc.appearance.head.clone())))
                .collect();
            let after_world = Instant::now();
            let title_drawn = if let Some(live) = live.as_mut() {
                live.draw_title(&mut client, &mut raster)?
            } else {
                false
            };
            let report = if title_drawn {
                crate::interface_render::Report::default()
            } else {
                renderer.draw_with_variables(
                    &mut client.scripts.source.cache,
                    &client.items,
                    &client.interfaces,
                    &layout,
                    &mut raster,
                    Some(&client.scripts.variables),
                )?
            };
            if let Some(lines) = live
                .as_ref()
                .filter(|_| !title_drawn)
                .and_then(|v| v.status_lines())
            {
                renderer.draw_status(&mut client.scripts.source.cache, &mut raster, &lines)?;
            }
            if !title_drawn {
                // GameClient.ix: full-canvas mouse cross after interfaces, before menus.
                raster.set_clip(0, 0, fit.logical[0], fit.logical[1]);
                match renderer.click_cross_sprites.draw(&mut client.scripts.source.cache, &mut raster,
                    client.click_cross, client.preferences.mouse_cross) {
                    Ok(()) | Err(crate::Error::PendingAsset { .. }) => {},
                    Err(error) => return Err(error),
                }
                if let Some(menu) = &client.menu {
                    raster.set_clip(0, 0, fit.logical[0], fit.logical[1]);
                    menu.draw(
                        &mut raster,
                        renderer.menu_font(&mut client.scripts.source.cache)?,
                        client.input.pointer,
                    )?;
                    if exercise {
                        std::fs::write("work/live-context-menu.bmp", raster.bmp()?)?;
                    }
                }
            }
            if exercise && client.selected_spell.is_some() {
                std::fs::write("work/live-spell-selection.bmp", raster.bmp()?)?;
            }
            if exercise && client.interfaces.drag_visual.is_some() {
                std::fs::write("work/live-widget-drag.bmp", raster.bmp()?)?;
            }
            if exercise && std::mem::take(&mut actions.map_capture) {std::fs::write("work/live-world-map.bmp",raster.bmp()?)?;}
            #[cfg(feature="private-test-adapter")]
            if let Some(adapter)=adapter.as_mut() {adapter.frame(&raster,&client,live.as_ref())?;}
            #[cfg(feature="native-profiling")]
            let before_present=presentation_trace.as_ref().filter(|_|drawable).map(|_|Instant::now());
            #[cfg(not(feature = "offscreen"))]
            let presentation_result=drawable.then(||presenter.submit(raster.pixels(), fit.logical,
                [rect.right, rect.bottom], fit.origin, fit.size));
            #[cfg(feature = "offscreen")]
            // #233 offscreen: nothing is presented (no window); the raster was already captured above.
            let presentation_result: Option<Result<()>> = None;
            #[cfg(feature="native-profiling")]
            let presentation_timing=before_present.map(|ui|(ui,Instant::now(),presentation_result.as_ref().is_some_and(|result|result.is_ok())));
            if let Some(Err(error)) = &presentation_result {
                presentation_errors += 1;
                if presentation_errors <= 3 || presentation_errors % 120 == 0 {
                    eprintln!("Native presentation failed ({presentation_errors}); retaining completed frame: {error}");
                }
            }
            #[cfg(feature="native-profiling")]
            if let Some((ui,done,success))=presentation_timing {
                if let Some(error)=crate::presentation_trace::Trace::complete(&mut presentation_trace,crate::presentation_trace::Sample {
                    times:[start,after_network,after_scripts,after_world,ui,done],logical:fit.logical,
                    physical:[rect.right,rect.bottom],pixels:raster.pixels(),success,events:presenter.events(),
                }) {eprintln!("Presentation trace disabled: {error}");}
            }
            if live.is_some() && start.elapsed() > Duration::from_millis(500) {
                println!(
                    "Slow native cycle {}: {:.3}s (network {:.3}, scripts {:.3}, world/audio {:.3}, UI {:.3})",
                    frames,
                    start.elapsed().as_secs_f64(),after_network.duration_since(start).as_secs_f64(),after_scripts.duration_since(after_network).as_secs_f64(),after_world.duration_since(after_scripts).as_secs_f64(),after_world.elapsed().as_secs_f64()
                );
            }
            client
                .frame_rate
                .frame(render_epoch.elapsed().as_millis() as u64 + 1);
            frames += 1;
            if std::mem::take(&mut editor_probe.capture) {std::fs::write("work/live-editor.bmp",raster.bmp()?)?;}
            if crate::native_smoke::blocked(smoke, live.as_ref().and_then(|live| live.logged_out())) {
                std::fs::write("work/live-native-failed.png", raster.png()?)?;
                return Err(crate::Error::Invalid("native smoke stopped at login/title prompt"));
            }
            let finished = drawable && (if exercise {
                actions.complete && (!inventory || editor_probe.complete)
            } else if let Some(live) = &live {
                crate::native_smoke::ready(live.elapsed(),live.observation_ready(&client),&client.data.base,report.pending.len(),title_drawn)
            } else {
                frames == 3
            });
            #[cfg(feature="native-profiling")]
            let finished=finished && zero_area_probe.is_none();
            #[cfg(feature="private-test-adapter")]
            let adapter_finished=adapter.as_ref().is_some_and(|adapter|adapter.stop_ready());
            #[cfg(not(feature="private-test-adapter"))]
            let adapter_finished=false;
            #[cfg(feature="native-profiling")]
            let benchmark_finished=if let (Some(run),Some((dir,settings)))=(&mut benchmark_run,&profile_directory) {
                let ready=finished && presentation_result.as_ref().is_some_and(|result|result.is_ok());
                let frame=presentation_trace.as_ref().ok_or(Error::Invalid("benchmark presentation trace unavailable"))?.frame_count();
                let event=run.observe(ready,frame);
                if event==crate::benchmark_run::Event::Warmed {
                    benchmark_rows.clear();
                    benchmark_start=Some(Raster::from_pixels(raster.width(),raster.height(),raster.pixels().to_vec())?);
                }
                if matches!(event,crate::benchmark_run::Event::Sample|crate::benchmark_run::Event::Complete{..}) {
                    let camera=live.as_ref().and_then(|live|live.capture_camera()).map_or("null".into(),|(ray,root)|format!("{{\"angles\":{:?},\"eye\":{:?},\"root_eye\":{root:?}}}",ray.angles,ray.eye));
                    let filter=live.as_ref().map_or("null".into(),|live|live.scene_filter_report.json());
                    benchmark_rows.push(format!("{{\"frame\":{frame},\"cycle\":{},\"heap_kib\":{},\"allocation_snapshot\":{:?},\"camera\":{camera},\"scene_filter\":{filter}}}",client.cycle,crate::client_metrics::heap_kib(),crate::client_metrics::allocation_snapshot()));
                }
                if let crate::benchmark_run::Event::Complete{first,last}=event {
                    let start=benchmark_start.take().ok_or(Error::Invalid("benchmark start frame missing"))?;
                    crate::study_profile::write_png_new(&dir.join("benchmark-start.png"),&start)?;
                    crate::study_profile::write_png_new(&dir.join("benchmark-end.png"),&raster)?;
                    if let Some(live)=live.as_ref() {std::fs::write(dir.join("benchmark-scene-filter.json"),live.scene_filter_report.json())?;}
                    std::fs::write(dir.join("benchmark-frames.jsonl"),benchmark_rows.join("\n"))?;
                    let js5=client.scripts.source.cache.recovery_stats().map_or("null".into(),|(a,b,c,d)|format!("[{a},{b},{c},{d}]"));
                    std::fs::write(dir.join("benchmark-window.json"),format!("{{\"first_frame\":{first},\"last_frame\":{last},\"samples\":{},\"readiness_resets\":{},\"headless\":{},\"position\":{:?},\"js5\":{js5}}}\n",benchmark_rows.len(),run.resets,settings.headless(),client.data.position))?;
                    true
                } else {false}
            } else {false};
            #[cfg(not(feature="native-profiling"))]
            let benchmark_finished=false;
            if smoke && finished || adapter_finished || benchmark_finished {
                // #181: the single journal finalization boundary. `stop_ready` held after this
                // frame's only poll (above), and the loop breaks below with no later poll.
                #[cfg(feature="private-test-adapter")]
                if adapter_finished { if let (Some(adapter),Some(live))=(adapter.as_mut(),live.as_mut()) { adapter.export_journal(live)?; } }
                if live.is_some() {
                    std::fs::write("work/live-native.png", raster.png()?)?;
                    let kinds = client
                        .zone
                        .pending
                        .iter()
                        .filter_map(|(_, record)| {
                            if let crate::zone::Record::Pending { kind, .. } = record {
                                Some(*kind)
                            } else {
                                None
                            }
                        })
                        .collect::<std::collections::BTreeSet<_>>();
                    println!("Native JS5 recovery (requests, completed, failures, retained bytes): {:?}", client.scripts.source.cache.recovery_stats());
                    println!("Native independent PCM: {:?}", audio_worker.stats());
                    println!("Native mouse recorder: {:?}", live.as_ref().and_then(|live| live.mouse_stats()));
                    println!("Native wheel/focus packets: {}", live.as_ref().map_or(0, |live| live.auxiliary_packets));
                    println!("Native keyboard packets: {}", live.as_ref().map_or(0, |live| live.keyboard_packets));
                    println!("Native mouse click packets: {}", live.as_ref().map_or(0, |live| live.mouse_click_packets));
        println!("Native simulation clock: {} game ticks; {} actor ticks",client.cycle,client.actor_cycle);
                    println!("Native zone state: {} ground piles; {} queued records, kinds {kinds:?}; server tick {}",client.zone.piles.len(),client.zone.pending.len(),client.server_tick);
                }
                println!(
                    "Native interface window: {frames} frames; {} widgets; {report:?}",
                    layout.widgets.len()
                );
                break;
            }
            std::thread::sleep(Duration::from_millis(20).saturating_sub(start.elapsed()));
        }
        // A failed capture is returned after the variable save below still runs.
        #[cfg(feature="native-profiling")]
        let capture=profile_directory.as_ref().map_or(Ok(()),|(dir,_)|presenter.save_completed_frame(&dir.join("presented.png")));
        if client.scripts.variables.varcs_dirty() {
            if let Some(saved) = varc_profile.as_mut() {
                if let Err(error) = client
                    .scripts
                    .variables
                    .encode_varcs()
                    .and_then(|bytes| saved.save(&bytes))
                {
                    eprintln!("Could not save native client variables on exit: {error}");
                }
            }
        }
        #[cfg(feature="native-profiling")]
        capture?;
        Ok(())
    }
    #[cfg(not(feature = "offscreen"))]
    pub fn run(cache_path: &str, zx: u16, zy: u16, plane: usize, smoke: bool) -> Result<()> {
        let mut cache = DiskCache::open(cache_path, 64 << 20)?;
        let scene = Scene::load_normal(
            &mut cache,
            &Normal {
                world_area: -1,
                zone_x: zx,
                zone_y: zy,
                players: None,
            },
        )?;
        study_view(cache,scene,plane,StudyOptions{smoke,..Default::default()})
    }
    #[cfg(not(feature = "offscreen"))]
    pub fn study_view(mut cache:DiskCache,scene:Scene,plane:usize,options:StudyOptions)->Result<()> {
        let smoke=options.smoke;
        #[cfg(feature="native-profiling")]
        if smoke && options.benchmark.is_some(){return Err(Error::Invalid("study smoke and benchmark are exclusive"));}
        #[cfg(feature="native-profiling")]
        let mut benchmark=options.benchmark.map(crate::study_profile::Recorder::new).transpose()?;
        #[cfg(feature="native-profiling")]
        let profiling=benchmark.is_some();
        #[cfg(not(feature="native-profiling"))]
        let profiling=false;
        if options.viewpoints.len()>4 || options.far.is_some_and(|far|!(50..=30000).contains(&far)) {
            return Err(Error::Invalid("offline study camera options"));
        }
        let resolve=|point|study_viewpoint([scene.base_x,scene.base_y],[scene.width,scene.height],point);
        let start=options.start.map(resolve).transpose()?;
        let viewpoints=options.viewpoints.iter().copied().map(resolve).collect::<Result<Vec<_>>>()?;
        #[cfg(feature="enhanced-renderer")]
        let mut enhanced=options.renderer.filter(|s|s.mode!=crate::enhanced::Mode::Classic)
            .map(crate::enhanced::Renderer::new).transpose().map_err(Error::Io)?;
        let mut mesh = WorldMesh::build(&scene, &mut cache, plane)?;
        #[cfg(feature="custom-assets")]
        if options.cutaway && mesh.pending_assets {return Err(Error::Invalid("authored study assets pending"));}

        let mut animations = crate::animation_runtime::Animations::load(&mut cache)?;
        let defs = crate::object::Definition::load_all(&mut cache)?;
        let mut textures = HashMap::<u16, Texture>::new();
        for (id, d) in Definition::load_all(&mut cache)? {
            textures.insert(id, Texture::load(&mut cache, d, 128, 0.8)?);
        }
        let palette = floor::palette(0.8)?;
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        let name = wide("OSRSRustViewport");
        let class = WndClass {
            size: std::mem::size_of::<WndClass>() as u32,
            style: 3,
            proc: Some(wndproc),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: 0,
            cursor: unsafe { LoadCursorW(0, 32512usize as *const u16) },
            background: 0,
            menu: ptr::null(),
            name: name.as_ptr(),
            icon_small: 0,
        };
        if unsafe { RegisterClassExW(&class) } == 0 {
            return Err(Error::Io(std::io::Error::last_os_error().to_string()));
        }
        let title =
            wide("OSRS 239 Rust â€” cache viewport | WASD move, arrows orbit, Q/E zoom, Esc close");
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                name.as_ptr(),
                title.as_ptr(),
                if smoke || profiling { 0x00cf0000 } else { 0x10cf0000 },
                i32::MIN,
                i32::MIN,
                1280,
                900,
                0,
                0,
                instance,
                ptr::null_mut(),
            )
        };
        if hwnd == 0 {
            return Err(Error::Io(std::io::Error::last_os_error().to_string()));
        }
        let _window_guard = Window(hwnd);
        let mut presenter = unsafe { crate::presentation::Presenter::new(hwnd) }?;
        let mut presentation_errors = 0u64;
        let mut frame = Frame::new(1280, 900)?;
        let (mut x, mut z, mut yaw, mut pitch, mut distance) = (
            52.0f32 * 128.0,
            52.0f32 * 128.0,
            -0.75f32,
            0.75f32,
            2600.0f32,
        );
        if let Some((position,range))=start {x=position[0];z=position[1];distance=range;}
        let (mut previous, mut last_title) = (Instant::now(), Instant::now());
        let mut last_animation = Instant::now();
        let mut frames = 0;
        let mut total_frames=0;
        'main: loop {
            let mut msg: Msg = unsafe { std::mem::zeroed() };
            while unsafe { PeekMessageW(&mut msg, 0, 0, 0, 1) } != 0 {
                if msg.message == 0x12 {
                    break 'main;
                }
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            #[cfg(feature="native-profiling")]
            let mut sample=crate::study_profile::Sample{period_ms:previous.elapsed().as_secs_f64()*1000.,..Default::default()};
            let dt = previous.elapsed().as_secs_f32().min(0.2);
            previous = Instant::now();
            let key = |k| unsafe { GetAsyncKeyState(k) < 0 };
            if !profiling && unsafe { GetForegroundWindow() } == hwnd {
                if key(0x1b) {
                    break 'main;
                }
                let speed = if key(0x10) { 1800.0 } else { 650.0 } * dt;
                let forward = key(0x57) as i32 - key(0x53) as i32;
                let side = key(0x44) as i32 - key(0x41) as i32;
                x += (forward as f32 * yaw.sin() + side as f32 * yaw.cos()) * speed;
                z += (forward as f32 * yaw.cos() - side as f32 * yaw.sin()) * speed;
                yaw += (key(0x27) as i32 - key(0x25) as i32) as f32 * dt;
                pitch += (key(0x26) as i32 - key(0x28) as i32) as f32 * dt;
                for (index,(position,range)) in viewpoints.iter().enumerate() {
                    if key(0x31+index as i32) {x=position[0];z=position[1];distance=*range;}
                }
                distance += (key(0x51) as i32 - key(0x45) as i32) as f32 * 1500.0 * dt;
            }
            x = x.clamp(128.0, 102.0 * 128.0);
            z = z.clamp(128.0, 102.0 * 128.0);
            pitch = pitch.clamp(0.15, 1.45);
            distance = distance.clamp(250.0, 12000.0);
            let mut rect: Rect = unsafe { std::mem::zeroed() };
            unsafe { GetClientRect(hwnd, &mut rect) };
            if rect.right <= 0 || rect.bottom <= 0 {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
            let (width, height) = (
                (rect.right as usize).min(1920),
                (rect.bottom as usize).min(1080),
            );
            if frame.width != width || frame.height != height {
                frame = Frame::new(width, height)?;
            }
            let [_,h,_] = study_target(&scene,plane,[x,z]);
            let horizontal = distance * pitch.cos();
            let mut camera = Camera::look_at(
                [
                    x - yaw.sin() * horizontal,
                    h - distance * pitch.sin(),
                    z - yaw.cos() * horizontal,
                ],
                [x, h, z],
                height as f32 * 0.84,
            )?;
            if let Some(far)=options.far {camera.far=far;}
            #[cfg(feature="native-profiling")]
            let pose_start=profiling.then(Instant::now);
            let ticks = (last_animation.elapsed().as_millis() / 20).min(50) as i32;
            if ticks > 0 {
                mesh.advance(&scene, &mut cache, &mut animations, ticks as u32)?;
                for texture in textures.values_mut() {
                    texture.animate(ticks);
                }
                last_animation = Instant::now();
            }
            #[cfg(feature="native-profiling")]
            {sample.pose_ms=pose_start.map_or(0.,|t|t.elapsed().as_secs_f64()*1000.);}
            #[cfg(feature="native-profiling")]
            let classic_start=profiling.then(Instant::now);
            frame.clear(0xff10151a);
            #[cfg(feature="custom-assets")]
            mesh.register_authored_art(&mut frame);
            #[cfg(feature="enhanced-renderer")]
            if enhanced.is_some(){frame.begin_presentation_capture();}
            #[cfg(feature="custom-assets")]
            if options.cutaway {
                mesh.update_environment_cutaway(&camera,[x,h,z]);
                frame.draw_iter(&camera,mesh.visible_triangles(plane as u8),&palette,&textures);
            } else {frame.draw(&camera,&mesh.triangles,&palette,&textures);}
            #[cfg(not(feature="custom-assets"))]
            frame.draw(&camera,&mesh.triangles,&palette,&textures);
            #[cfg(feature="native-profiling")]
            {sample.classic_ms=classic_start.map_or(0.,|t|t.elapsed().as_secs_f64()*1000.);}
            #[cfg(feature="enhanced-renderer")]
            if let Some(renderer)=&mut enhanced {
                #[cfg(feature="native-profiling")]
                let enhanced_start=profiling.then(Instant::now);
                let geometry=frame.take_presentation_geometry()?.ok_or(Error::Invalid("offline capture absent"))?;
                let triangles=geometry.iter().collect::<Vec<_>>();
                #[cfg(feature="custom-assets")] let cutaway=options.cutaway;
                #[cfg(not(feature="custom-assets"))] let cutaway=false;
                let casters=if renderer.settings().shadows==0 {Vec::new()}
                    else if cutaway {crate::enhanced::root_shadow_casters(mesh.visible_triangles(plane as u8))}
                    else {crate::enhanced::root_shadow_casters(mesh.triangles.iter())};
                frame.pixels=renderer.render_with_shadow_casters(crate::enhanced::SceneFrame{camera:&camera,triangles:&triangles,stable_count:0,stable_revision:None,albedo:None,palette:&palette,textures:&textures,size:[width as u32,height as u32],light_focus:[x,h,z],ambient:64,contrast:768},None,&casters).map_err(Error::Io)?;
                #[cfg(feature="native-profiling")]
                {sample.enhanced_ms=enhanced_start.map_or(0.,|t|t.elapsed().as_secs_f64()*1000.);
                 sample.backend=[Some(renderer.stats.prepare_ms),Some(renderer.stats.render_ms),Some(renderer.stats.readback_ms),renderer.stats.gpu_ms];}
            }
            #[cfg(feature="native-profiling")]
            let presentation_start=profiling.then(Instant::now);
            if let Err(error) = presenter.submit(&frame.pixels, [width as i32, height as i32],
                [rect.right, rect.bottom], [0, 0], [rect.right, rect.bottom]) {
                presentation_errors += 1;
                if presentation_errors <= 3 || presentation_errors % 120 == 0 {
                    eprintln!("Native presentation failed ({presentation_errors}); retaining completed frame: {error}");
                }
            }
            #[cfg(feature="native-profiling")]
            if let Some(recorder)=&mut benchmark {
                sample.presentation_ms=presentation_start.map_or(0.,|t|t.elapsed().as_secs_f64()*1000.);
                if recorder.record(sample,presentation_errors==0)? {
                    #[cfg(feature="enhanced-renderer")]
                    let backend=enhanced.as_ref().map_or_else(||"classic".into(),|r|r.stats.json());
                    #[cfg(not(feature="enhanced-renderer"))]
                    let backend=String::from("classic");
                    let details=format!("plane={plane}\nscene_base={},{}\nsize={width}x{height}\ncamera={:?}\ntarget={:?}\nfocal={}\nfar={}\nanimations=realtime\npresentation_errors={presentation_errors}\nbackend={backend}\n",scene.base_x,scene.base_y,camera.position,[x,h,z],camera.focal,camera.far);
                    recorder.finish(&frame.raster()?,&details)?;
                    println!("Offline native window benchmark: 160 completed frames, 20 warmup, 140 measured; no gameplay");
                    break 'main;
                }
            }
            frames += 1;total_frames+=1;
            if study_smoke_finished(smoke,total_frames,presentation_errors)? {
                println!(
                    "Native Win32 viewport smoke test: two frames, {} triangles, {}x{} pixels",
                    frame.triangles, frame.width, frame.height
                );
                break 'main;
            }
            if last_title.elapsed() >= Duration::from_millis(500) {
                let mut cursor = Point { x: 0, y: 0 };
                unsafe {
                    GetCursorPos(&mut cursor);
                    ScreenToClient(hwnd, &mut cursor);
                }
                let mut hover = String::new();
                if cursor.x >= 0 && cursor.y >= 0 && cursor.x < rect.right && cursor.y < rect.bottom
                {
                    let tag = frame.tags[(cursor.y as usize * height / rect.bottom as usize)
                        * width
                        + cursor.x as usize * width / rect.right as usize];
                    if tag != 0 && tag >> 63 == 0 {
                        let id = ((tag >> 32) - 1) as u32;
                        hover = defs
                            .get(&id)
                            .map(|d| format!(" | {} ({id})", d.name))
                            .unwrap_or_default();
                    }
                }
                let text = wide(&format!(
                    "OSRS 239 Rust â€” cache viewport | {:.1} fps | {},{} | WASD, arrows, Q/E{}",
                    frames as f32 / last_title.elapsed().as_secs_f32(),
                    scene.base_x + (x as i32 >> 7),
                    scene.base_y + (z as i32 >> 7),
                    hover
                ));
                unsafe { SetWindowTextW(hwnd, text.as_ptr()) };
                frames = 0;
                last_title = Instant::now();
            }
            let remaining = Duration::from_millis(16).saturating_sub(previous.elapsed());
            if !remaining.is_zero() {
                std::thread::sleep(remaining);
            }
        }
        #[cfg(feature="native-profiling")]
        if benchmark.as_ref().is_some_and(|b|!b.completed()){return Err(Error::Invalid("study benchmark interrupted"));}
        Ok(())
    }
}
#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::interface_view;
#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::run;

#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::live_view;

#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::live_token_view;

#[cfg(all(windows,feature = "custom-assets", not(feature = "offscreen")))]
pub use windows::live_view_with_cache;

#[cfg(all(windows,feature="private-test-adapter"))]
pub use windows::private_authored_test;

#[cfg(all(windows,feature="native-profiling",feature="custom-assets"))]
pub use windows::profiled_authored_live;

#[cfg(all(windows,feature="enhanced-renderer",feature="custom-assets"))]
pub use windows::enhanced_authored_live;
#[cfg(all(windows,feature="native-profiling",feature="enhanced-renderer",feature="custom-assets"))]
pub use windows::profiled_enhanced_authored_live;
#[cfg(all(windows,feature="enhanced-renderer",feature="private-test-adapter"))]
pub use windows::enhanced_private_authored_test;

#[cfg(all(windows, not(feature = "offscreen")))]
pub use windows::study_view;
