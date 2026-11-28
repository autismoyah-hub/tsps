//! Explicit immutable terrain package; maintained native loop and private menu adapter.
//! Hidden by default (window created without WS_VISIBLE, as private-authored-test
//! --headless). `--visible` restores the source launcher's visible window.
#[cfg(windows)]
fn main()->Result<(),Box<dyn std::error::Error>>{
 use gielinor_client::{asset_package::Package,disk_cache::DiskCache,enhanced::Settings};
 let a=std::env::args().skip(1).collect::<Vec<_>>();if a.len()<10{return Err("PACKAGE SHA256 CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY COMMAND_DIRECTORY MODE ADAPTER [--headless|--visible] [renderer options]".into());}
 let (headless,options)=window_policy(&a[10..])?;
 if a[1].len()!=64{return Err("SHA256 length".into());}let mut pin=[0;32];for(i,v)in pin.iter_mut().enumerate(){*v=u8::from_str_radix(&a[1][i*2..i*2+2],16)?;}
 let settings=Settings::from_launch_args(&a[8],&a[9],&options)?;
 let p=Package::read(std::path::Path::new(&a[0]))?;let mut c=DiskCache::open_remote(&a[2],64<<20,a[3].parse()?)?;c.wait_for_recovery(true);c.mount_package(p,pin)?;
 if headless{visibility::watch();}
 gielinor_client::viewport::enhanced_private_authored_test(c,&a[3],&a[4],&a[5],&a[6],a[7].clone().into(),settings,headless)?;Ok(())
}
#[cfg(not(windows))]
fn main(){panic!("Windows required");}

/// Splits the window policy from renderer options. Headless unless `--visible`.
#[cfg_attr(not(windows),allow(dead_code))]
fn window_policy(options:&[String])->Result<(bool,Vec<String>),String>{
 let policy=options.iter().filter(|o|matches!(o.as_str(),"--headless"|"--visible")).collect::<Vec<_>>();
 if policy.len()>1{return Err("Use at most one of --headless/--visible".into());}
 Ok((policy.first().is_none_or(|o|*o=="--headless"),options.iter().filter(|o|!matches!(o.as_str(),"--headless"|"--visible")).cloned().collect()))
}

/// Read-only self-check: enumerates top-level windows and exits at once if any
/// window owned by this process becomes visible. Never moves, focuses or sends
/// input to any window.
#[cfg(windows)]
mod visibility{
 type H=isize;
 #[link(name="user32")]
 unsafe extern "system"{
  fn EnumWindows(callback:extern "system" fn(H,isize)->i32,parameter:isize)->i32;
  fn GetWindowThreadProcessId(hwnd:H,process:*mut u32)->u32;
  fn IsWindowVisible(hwnd:H)->i32;
 }
 struct Found{process:u32,windows:Vec<(H,bool)>}
 extern "system" fn visit(hwnd:H,parameter:isize)->i32{
  let found=unsafe{&mut *(parameter as *mut Found)};let mut process=0;
  unsafe{GetWindowThreadProcessId(hwnd,&mut process);}
  if process==found.process{found.windows.push((hwnd,unsafe{IsWindowVisible(hwnd)}!=0));}
  1
 }
 /// Top-level windows of this process and whether each is visible. `visit`
 /// never stops enumeration early, so a zero EnumWindows result is a failure.
 pub fn windows()->std::io::Result<Vec<(H,bool)>>{
  let mut found=Found{process:std::process::id(),windows:Vec::new()};
  if unsafe{EnumWindows(visit,&mut found as *mut Found as isize)}==0{return Err(std::io::Error::last_os_error());}
  Ok(found.windows)
 }
 /// Owned and visible top-level window counts.
 pub fn owned()->std::io::Result<[u32;2]>{
  let windows=windows()?;
  Ok([windows.len() as u32,windows.iter().filter(|(_,visible)|*visible).count() as u32])
 }
 pub fn watch(){
  std::thread::spawn(||{
   let mut reported=false;
   loop{
    let [owned,visible]=match owned(){Ok(count)=>count,Err(error)=>{eprintln!("headless window check FAILED: EnumWindows: {error}; exiting");std::process::exit(70);}};
    if visible!=0{eprintln!("headless window check FAILED: {visible} of {owned} owned top-level windows visible; exiting");std::process::exit(70);}
    if owned!=0&&!reported{eprintln!("headless window check: {owned} owned top-level windows, 0 visible");reported=true;}
    std::thread::sleep(std::time::Duration::from_millis(50));
   }
  });
 }
}

#[cfg(test)]
mod tests{
 use super::window_policy;
 fn args(v:&[&str])->Vec<String>{v.iter().map(|s|s.to_string()).collect()}
 #[test]fn headless_is_the_default_and_renderer_options_pass_through(){
  assert_eq!(window_policy(&[]).unwrap(),(true,vec![]));
  assert_eq!(window_policy(&args(&["--scale=1","--headless","--aa=on"])).unwrap(),(true,args(&["--scale=1","--aa=on"])));
  assert_eq!(window_policy(&args(&["--visible","--scale=1"])).unwrap(),(false,args(&["--scale=1"])));
 }
 #[test]fn conflicting_or_repeated_window_policies_are_rejected(){
  for v in[&["--headless","--visible"][..],&["--headless","--headless"],&["--visible","--visible"]]{assert!(window_policy(&args(v)).is_err());}
 }
 #[cfg(windows)]
 #[test]fn visibility_check_sees_a_hidden_owned_window_as_not_visible(){
  #[link(name="user32")]
  unsafe extern "system"{
   fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:isize,menu:isize,instance:isize,parameter:*mut std::ffi::c_void)->isize;
   fn DestroyWindow(hwnd:isize)->i32;
  }
  let class:Vec<u16>="STATIC\0".encode_utf16().collect();
  assert_eq!(super::visibility::owned().unwrap()[1],0,"test process starts with no visible windows");
  let hwnd=unsafe{CreateWindowExW(0,class.as_ptr(),class.as_ptr(),0x00cf0000,-10000,-10000,64,64,0,0,0,std::ptr::null_mut())};
  assert_ne!(hwnd,0);
  // The first window may also create hidden helper windows (Default IME), so
  // check the exact window rather than a process-wide total.
  let windows=super::visibility::windows().unwrap();
  assert_eq!(windows.iter().filter(|(h,_)|*h==hwnd).collect::<Vec<_>>(),[&(hwnd,false)]);
  assert_eq!(super::visibility::owned().unwrap()[1],0);
  unsafe{DestroyWindow(hwnd);}
 }
}
