//! #233 offscreen (feature `offscreen`): the OS-facing sites compile to their existing
//! non-Windows fallbacks, so no window, clipboard, shell, WinHTTP or GDI+ code is built.
//! These tests pin each site's actual offscreen outcome. The null audio device has its own
//! tests (`audio_output`, `audio_worker`).
use crate::Error;

#[test]
fn offscreen_clipboard_read_is_none_and_write_refused() {
    assert_eq!(crate::clipboard::read(4096), None);
    assert!(!crate::clipboard::write(&[u16::from(b'a')]));
}

#[test]
fn offscreen_open_web_url_is_refused_after_the_unchanged_url_checks() {
    // live_runtime reports this error as "external URL: ..." and opens nothing.
    assert_eq!(crate::remote_input::open_web_url("https://example.com/"), Err(Error::Unsupported("native browser platform")));
    assert_eq!(crate::remote_input::open_web_url("mailto:someone@example.com"), Err(Error::Unsupported("external URL scheme")));
    assert_eq!(crate::remote_input::open_web_url("not a url"), Err(Error::Invalid("external URL")));
}

#[test]
fn offscreen_world_http_fetch_post_json_and_rest_are_refused() {
    assert_eq!(
        crate::world_http::fetch("http://127.0.0.1:1/worlds").map(|_| ()),
        Err(Error::Invalid("native world-list HTTP is unavailable on this platform"))
    );
    assert_eq!(
        crate::world_http::post_json("http://127.0.0.1:1/crm").map(|_| ()),
        Err(Error::Invalid("native REST HTTP unavailable"))
    );
    let request = crate::world_http::RestRequest { method: "GET", headers: Vec::new(), body: Vec::new() };
    assert_eq!(
        crate::world_http::rest("http://127.0.0.1:1/rest", &request).map(|_| ()),
        Err(Error::Invalid("native REST HTTP unavailable"))
    );
    // The request validation before the transport is unchanged.
    let bad = crate::world_http::RestRequest { method: "PUT", headers: Vec::new(), body: Vec::new() };
    assert_eq!(crate::world_http::rest("http://127.0.0.1:1/rest", &bad).map(|_| ()), Err(Error::Invalid("REST request method or size")));
}

#[test]
fn offscreen_image_decode_is_refused() {
    // Title background: absent (below). Remote interface images: `.ok()`, so absent.
    // World map region/overview images: this error propagates (the run ends in error).
    let png = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    assert_eq!(crate::image_decode::decode(&png, 1 << 20).map(|_| ()), Err(Error::Unsupported("native image decoder on this platform")));
}

#[test]
fn offscreen_title_assets_load_without_background_and_keep_all_other_assets() {
    fn chunk<'a>(r: &mut crate::buffer::Reader<'a>) -> &'a [u8] {
        let n = r.u32().unwrap() as usize;
        r.bytes(n).unwrap()
    }
    let mut a = crate::buffer::Reader::new(include_bytes!("../tests/fixtures/title-assets.bin"));
    let image = chunk(&mut a);
    let sprites = [chunk(&mut a), chunk(&mut a), chunk(&mut a)];
    let font = [chunk(&mut a), chunk(&mut a)];
    assert_eq!(a.remaining(), 0);
    let assets = crate::title::Assets::decode(image, sprites, font).unwrap();
    let mut raster = crate::raster::Raster::new(765, 503, 16 << 20).unwrap();
    assets.background(&mut raster).unwrap();
    // The background (and its mirror) would cover the top-left corner; absent, it stays clear,
    // while the logo and panel sprites still draw.
    assert_eq!(raster.pixels()[0], 0);
    assert!(raster.pixels().iter().any(|&p| p != 0), "logo and panel drawn");
}
