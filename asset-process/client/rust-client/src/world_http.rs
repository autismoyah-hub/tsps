//! Native HTTP world-list transport; di.as uses five-second timeouts and a revision User-Agent.
//! WinHTTP API contracts: https://learn.microsoft.com/windows/win32/winhttp/winhttp-functions
use crate::{Error, Result};
pub fn fetch(url: &str) -> Result<Vec<u8>> {
    #[cfg(all(windows, not(feature = "offscreen")))]
    {
        native::fetch(url)
    }
    #[cfg(any(not(windows), feature = "offscreen"))]
    {
        let _ = url;
        Err(Error::Invalid(
            "native world-list HTTP is unavailable on this platform",
        ))
    }
}
/// Native REST response. CRM keeps HTTP error responses for its status checks.
#[derive(Clone, Debug)]
pub struct Response { pub status: u32, pub body: Vec<u8> }
/// Send the empty JSON POST used by the original CRM session API. Redirects are disabled.
pub fn post_json(url: &str) -> Result<Response> {
    #[cfg(all(windows, not(feature = "offscreen")))] { native::post_json(url) }
    #[cfg(any(not(windows), feature = "offscreen"))] { let _ = url; Err(Error::Invalid("native REST HTTP unavailable")) }
}
/// Authenticated REST requests share the native transport, without redirects.
/// Deliberately not Debug: headers and bodies may contain account secrets.
pub struct RestRequest {
    pub method: &'static str,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
pub struct RestResponse {
    pub status: u32,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
pub fn rest(url: &str, request: &RestRequest) -> Result<RestResponse> {
    if !matches!(request.method, "GET" | "POST") || request.body.len() > 8 << 20 {
        return Err(Error::Invalid("REST request method or size"));
    }
    for (name, value) in &request.headers {
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || value.contains(['\r', '\n', '\0']) {
            return Err(Error::Invalid("REST request header"));
        }
    }
    #[cfg(all(windows, not(feature = "offscreen")))] { native::rest(url, request) }
    #[cfg(any(not(windows), feature = "offscreen"))] { let _ = url; Err(Error::Invalid("native REST HTTP unavailable")) }
}
// java.net.URL resolves relative paths, but preserves dot segments in absolute
// paths and leading /../ segments above the root. Query-only redirects use the
// base directory, unlike the common URI-resolution algorithm.
fn redirect_url(base: &str, location: &str) -> Result<String> {
    let mut location = location.trim_matches(|c: char| c <= ' ');
    if location
        .get(..4)
        .is_some_and(|s| s.eq_ignore_ascii_case("url:"))
    {
        location = &location[4..];
    }
    let scheme = base
        .find("://")
        .ok_or(Error::Invalid("world-list base URL"))?;
    let host_start = scheme + 3;
    let path_start = base[host_start..]
        .find(['/', '?', '#'])
        .map_or(base.len(), |i| host_start + i);
    let origin = &base[..path_start];
    let base_path = base[path_start..].split(['?', '#']).next().unwrap_or("");
    if !location.starts_with('#') {
        if let Some(colon) = location.find(':').filter(|&i| !location[..i].contains('/')) {
            let protocol = &location[..colon];
            if protocol
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
                && protocol
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"+-.".contains(&b))
            {
                if protocol.eq_ignore_ascii_case(&base[..scheme]) && base_path.starts_with('/') {
                    location = &location[colon + 1..];
                } else {
                    let tail = &location[colon + 1..];
                    let tail = if tail.starts_with("//")
                        && !tail.starts_with("////")
                        && (tail.len() == 2 || matches!(tail.as_bytes()[2], b'/' | b'?' | b'#'))
                    {
                        &tail[2..]
                    } else {
                        tail
                    };
                    return Ok(format!("{}:{tail}", protocol.to_ascii_lowercase()));
                }
            }
        }
    }
    if location.starts_with("//")
        && (location.len() == 2 || matches!(location.as_bytes()[2], b'?' | b'#'))
    {
        return Ok(format!("{}:{base_path}{}", &base[..scheme], &location[2..]));
    }
    if location.starts_with("//") && !location.starts_with("////") {
        return Ok(format!(
            "{}:{}",
            &base[..scheme],
            if location.starts_with("///") {
                &location[2..]
            } else {
                location
            }
        ));
    }
    if location.is_empty() {
        return Ok(base.into());
    }
    if location.starts_with('#') {
        return Ok(format!("{}{location}", base.split('#').next().unwrap()));
    }
    if location.starts_with('/') {
        return Ok(format!("{origin}{location}"));
    }
    let suffix_at = location.find(['?', '#']).unwrap_or(location.len());
    let directory = base_path.rfind('/').map_or("/", |i| &base_path[..i + 1]);
    let mut path = format!("{directory}{}", &location[..suffix_at]);
    // URLStreamHandler only normalizes a relative path when the context has a path.
    // Its historical slash/dot rules deliberately differ from RFC URI normalization.
    if !base_path.is_empty() && suffix_at > 0 {
        while let Some(i) = path.find("/./") {
            path.replace_range(i..i + 2, "");
        }
        let mut start = 0;
        while let Some(i) = path[start..].find("/../").map(|i| start + i) {
            let previous = path[..i].rfind('/');
            if let Some(previous) = previous
                .filter(|&previous| path[previous..].find("/../").map(|n| previous + n) != Some(0))
            {
                path.replace_range(previous..i + 3, "");
                start = 0;
            } else {
                start = i + 3;
            }
        }
        while path.ends_with("/..") {
            let i = path.find("/..").unwrap();
            if let Some(previous) = path[..i].rfind('/') {
                path.truncate(previous + 1);
            } else {
                break;
            }
        }
        if path.starts_with("./") && path.len() > 2 {
            path.drain(..2);
        }
        if path.ends_with("/.") {
            path.pop();
        }
    }
    Ok(format!("{origin}{path}{}", &location[suffix_at..]))
}
#[cfg(all(windows, not(feature = "offscreen")))]
mod native {
    use super::*;
    use std::{
        ffi::c_void,
        ptr::{null, null_mut},
        time::{Duration, Instant},
    };
    type Handle = *mut c_void;
    #[repr(C)]
    struct Components {
        size: u32,
        scheme: *mut u16,
        scheme_len: u32,
        scheme_id: i32,
        host: *mut u16,
        host_len: u32,
        port: u16,
        user: *mut u16,
        user_len: u32,
        password: *mut u16,
        password_len: u32,
        path: *mut u16,
        path_len: u32,
        extra: *mut u16,
        extra_len: u32,
    }
    #[link(name = "winhttp")]
    extern "system" {
        fn WinHttpCrackUrl(url: *const u16, len: u32, flags: u32, parts: *mut Components) -> i32;
        fn WinHttpOpen(
            agent: *const u16,
            access: u32,
            proxy: *const u16,
            bypass: *const u16,
            flags: u32,
        ) -> Handle;
        fn WinHttpSetTimeouts(
            session: Handle,
            resolve: i32,
            connect: i32,
            send: i32,
            receive: i32,
        ) -> i32;
        fn WinHttpConnect(session: Handle, host: *const u16, port: u16, reserved: u32) -> Handle;
        fn WinHttpOpenRequest(
            connection: Handle,
            verb: *const u16,
            path: *const u16,
            version: *const u16,
            referer: *const u16,
            accept: *const *const u16,
            flags: u32,
        ) -> Handle;
        fn WinHttpSetOption(handle: Handle, option: u32, buffer: *mut c_void, len: u32) -> i32;
        fn WinHttpSendRequest(
            request: Handle,
            headers: *const u16,
            header_len: u32,
            body: *mut c_void,
            body_len: u32,
            total_len: u32,
            context: usize,
        ) -> i32;
        fn WinHttpReceiveResponse(request: Handle, reserved: *mut c_void) -> i32;
        fn WinHttpQueryHeaders(
            request: Handle,
            level: u32,
            name: *const u16,
            buffer: *mut c_void,
            len: *mut u32,
            index: *mut u32,
        ) -> i32;
        fn WinHttpReadData(request: Handle, buffer: *mut c_void, len: u32, read: *mut u32) -> i32;
        fn WinHttpCloseHandle(handle: Handle) -> i32;
    }
    struct Owned(Handle);
    impl Drop for Owned {
        fn drop(&mut self) {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }
    fn check(success: i32) -> Result<()> {
        if success == 0 {
            Err(Error::Io(format!(
                "world-list HTTP: {}",
                std::io::Error::last_os_error()
            )))
        } else {
            Ok(())
        }
    }
    fn own(handle: Handle) -> Result<Owned> {
        check(i32::from(!handle.is_null()))?;
        Ok(Owned(handle))
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    // CrackUrl points into the input allocation. Empty components may have null pointers.
    unsafe fn component(pointer: *const u16, len: u32) -> Vec<u16> {
        if len == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(pointer, len as usize).to_vec()
        }
    }
    pub fn fetch(url: &str) -> Result<Vec<u8>> {
        fetch_inner(url, 0, Instant::now(), None, None).map(|response| response.body)
    }
    pub fn post_json(url: &str) -> Result<super::Response> {
        let request = super::RestRequest { method: "POST", headers: vec![("Content-Type".into(), "application/json".into())], body: b"{}".to_vec() };
        rest(url, &request).map(|r| super::Response { status: r.status, body: r.body })
    }
    pub fn rest(url: &str, request: &super::RestRequest) -> Result<super::RestResponse> {
        fetch_inner(url, 0, Instant::now(), None, Some(request))
    }
    fn fetch_inner(
        url: &str,
        redirects: usize,
        started: Instant,
        endpoint: Option<&str>,
        rest: Option<&super::RestRequest>,
    ) -> Result<super::RestResponse> {
        if started.elapsed() > Duration::from_secs(30) {
            return Err(Error::Invalid("world-list HTTP total timeout"));
        }
        if url.len() > 16384 || url.contains('\0') {
            return Err(Error::Invalid("world-list URL"));
        }
        let base = url.split('#').next().unwrap_or(url);
        let url = wide(base);
        let mut c: Components = unsafe { std::mem::zeroed() };
        c.size = std::mem::size_of::<Components>() as u32;
        c.host_len = 1;
        c.path_len = 1;
        c.extra_len = 1;
        c.user_len = 1;
        c.password_len = 1;
        unsafe {
            check(WinHttpCrackUrl(
                url.as_ptr(),
                (url.len() - 1) as u32,
                0,
                &mut c,
            ))?;
            if !matches!(c.scheme_id, 1 | 2) || c.host_len == 0 {
                return Err(Error::Invalid("world-list HTTP URL scheme/host"));
            }
            if c.user_len != 0 || c.password_len != 0 {
                return Err(Error::Invalid("world-list URL credentials are unsupported"));
            }
            let mut host = component(c.host, c.host_len);
            host.push(0);
            let mut path = component(c.path, c.path_len);
            let path_text = String::from_utf16(&path)
                .map_err(|_| Error::Invalid("world-list URL path encoding"))?;
            let mut depth = 0;
            for segment in path_text.split('/') {
                match segment {
                    "." => {
                        return Err(Error::Invalid(
                            "world-list native HTTP cannot preserve URL dot segments",
                        ))
                    }
                    ".." if depth > 0 => {
                        return Err(Error::Invalid(
                            "world-list native HTTP cannot preserve URL dot segments",
                        ))
                    }
                    "" | ".." => (),
                    _ => depth += 1,
                }
            }
            if path.is_empty() {
                path.push(b'/' as u16);
            }
            path.extend(component(c.extra, c.extra_len));
            path.push(0);
            let session = own(WinHttpOpen(
                wide("OldSchoolRuneScape/239").as_ptr(),
                0,
                null(),
                null(),
                0,
            ))?;
            check(WinHttpSetTimeouts(session.0, 5000, 5000, 5000, 5000))?;
            // HttpURLConnection's 305 path connects to Location but retains the
            // original path and Host. A subsequent ordinary redirect resets routing.
            let mut routed_host = Vec::new();
            let mut routed_port = c.port;
            if let Some(endpoint) = endpoint {
                let route_url = wide(endpoint);
                let mut route: Components = std::mem::zeroed();
                route.size = std::mem::size_of::<Components>() as u32;
                route.host_len = 1;
                route.user_len = 1;
                route.password_len = 1;
                check(WinHttpCrackUrl(
                    route_url.as_ptr(),
                    (route_url.len() - 1) as u32,
                    0,
                    &mut route,
                ))?;
                if route.scheme_id != 1
                    || route.host_len == 0
                    || route.user_len != 0
                    || route.password_len != 0
                {
                    return Err(Error::Invalid("world-list HTTP proxy redirect endpoint"));
                }
                routed_host = component(route.host, route.host_len);
                routed_host.push(0);
                routed_port = route.port;
            }
            let connection = own(WinHttpConnect(
                session.0,
                if endpoint.is_some() {
                    routed_host.as_ptr()
                } else {
                    host.as_ptr()
                },
                routed_port,
                0,
            ))?;
            let request = own(WinHttpOpenRequest(
                connection.0,
                wide(rest.map_or("GET", |r| r.method)).as_ptr(),
                path.as_ptr(),
                null(),
                null(),
                null(),
                (if c.scheme_id == 2 { 0x00800000 } else { 0 }) | 0x40,
            ))?;
            // This stateless list request has no cookie jar or account authentication.
            // Handle redirects explicitly: Java reads a 302 body if Location is absent,
            // and does not follow redirects between HTTP and HTTPS.
            let mut disabled = 1u32 | 2 | 4;
            check(WinHttpSetOption(
                request.0,
                63,
                &mut disabled as *mut _ as *mut c_void,
                4,
            ))?;
            let mut headers = String::from(
                "Connection: close\r\nCache-Control: no-cache\r\nPragma: no-cache\r\n",
            );
            if endpoint.is_some() {
                let mut authority = String::from_utf16(&host[..host.len() - 1])
                    .map_err(|_| Error::Invalid("world-list HTTP host encoding"))?;
                if authority.contains(':') && !authority.starts_with('[') {
                    authority = format!("[{authority}]");
                }
                if c.port != 80 {
                    authority.push_str(&format!(":{}", c.port));
                }
                headers.push_str(&format!("Host: {authority}\r\n"));
            }
            if let Some(rest) = rest {
                for (name, value) in &rest.headers { headers.push_str(&format!("{name}: {value}\r\n")); }
            }
            let headers = wide(&headers);
            let body = rest.map_or(&[][..], |r| r.body.as_slice());
            check(WinHttpSendRequest(
                request.0,
                headers.as_ptr(),
                (headers.len() - 1) as u32,
                if body.is_empty() { null_mut() } else { body.as_ptr() as *mut c_void },
                body.len() as u32,
                body.len() as u32,
                0,
            ))?;
            check(WinHttpReceiveResponse(request.0, null_mut()))?;
            let mut status = 0u32;
            let mut size = 4;
            check(WinHttpQueryHeaders(
                request.0,
                19 | 0x20000000,
                null(),
                &mut status as *mut _ as *mut c_void,
                &mut size,
                null_mut(),
            ))?;
            if rest.is_none() && status >= 400 {
                return Err(Error::Io(format!("world-list HTTP status {status}")));
            }
            if rest.is_none() && matches!(status, 300 | 301 | 302 | 303 | 305 | 307) {
                let mut size = 0;
                WinHttpQueryHeaders(request.0, 33, null(), null_mut(), &mut size, null_mut());
                if size > 0 {
                    if size > 32768 {
                        return Err(Error::Invalid("world-list redirect URL length"));
                    }
                    let mut location = vec![0u16; (size as usize + 1) / 2];
                    check(WinHttpQueryHeaders(
                        request.0,
                        33,
                        null(),
                        location.as_mut_ptr() as *mut c_void,
                        &mut size,
                        null_mut(),
                    ))?;
                    let length = location
                        .iter()
                        .position(|&v| v == 0)
                        .unwrap_or(location.len());
                    let location = String::from_utf16(&location[..length])
                        .map_err(|_| Error::Invalid("world-list redirect encoding"))?;
                    let next = redirect_url(base, &location)?;
                    let combined = wide(&next);
                    let mut target: Components = std::mem::zeroed();
                    target.size = std::mem::size_of::<Components>() as u32;
                    check(WinHttpCrackUrl(
                        combined.as_ptr(),
                        (combined.len() - 1) as u32,
                        0,
                        &mut target,
                    ))?;
                    if target.scheme_id == c.scheme_id {
                        if redirects >= 19 {
                            return Err(Error::Invalid("world-list redirect limit"));
                        }
                        drop(request);
                        drop(connection);
                        drop(session);
                        if status == 305 {
                            if c.scheme_id != 1 {
                                return Err(Error::Invalid(
                                    "world-list HTTPS proxy redirect unsupported",
                                ));
                            }
                            return fetch_inner(base, redirects + 1, started, Some(&next), None);
                        }
                        return fetch_inner(&next, redirects + 1, started, None, None);
                    }
                }
            }
            let mut length = 0u32;
            size = 4;
            let has_length = WinHttpQueryHeaders(
                request.0,
                5 | 0x20000000,
                null(),
                &mut length as *mut _ as *mut c_void,
                &mut size,
                null_mut(),
            ) != 0;
            if has_length && length > 8 << 20 {
                return Err(Error::Invalid("world-list size limit"));
            }
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                if started.elapsed() > Duration::from_secs(30) {
                    return Err(Error::Invalid("world-list HTTP total timeout"));
                }
                let mut read = 0;
                check(WinHttpReadData(
                    request.0,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut read,
                ))?;
                if read == 0 {
                    break;
                }
                if bytes.len() + read as usize > 8 << 20 {
                    return Err(Error::Invalid("world-list size limit"));
                }
                bytes.extend_from_slice(&buffer[..read as usize]);
            }
            if has_length && bytes.len() != length as usize {
                return Err(Error::Invalid("truncated world-list HTTP body"));
            }
            let mut response_headers = Vec::new();
            if rest.is_some() {
                let mut size = 0;
                WinHttpQueryHeaders(request.0, 22, null(), null_mut(), &mut size, null_mut());
                if size > 65536 { return Err(Error::Invalid("REST response header size")); }
                if size > 0 {
                    let mut raw = vec![0u16; (size as usize + 1) / 2];
                    check(WinHttpQueryHeaders(request.0, 22, null(), raw.as_mut_ptr() as *mut c_void, &mut size, null_mut()))?;
                    let raw = String::from_utf16_lossy(&raw);
                    for line in raw.trim_end_matches('\0').split("\r\n") {
                        if let Some((name, value)) = line.split_once(':') {
                            response_headers.push((name.to_owned(), value.trim().to_owned()));
                        }
                    }
                }
            }
            Ok(super::RestResponse { status, headers: response_headers, body: bytes })
        }
    }
}

#[cfg(test)]
mod redirect_tests {
    use super::*;
    use crate::buffer::Reader;
    fn string(r: &mut Reader<'_>) -> String {
        let n = r.u16().unwrap() as usize;
        String::from_utf8(r.bytes(n).unwrap().to_vec()).unwrap()
    }
    #[test]
    fn original_java_redirects() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/world-redirect-jvm.bin"));
        let count = r.u32().unwrap();
        let mut failures = 0;
        for case in 0..count {
            let base = string(&mut r);
            let location = string(&mut r);
            let expected = if r.u8().unwrap() != 0 {
                Some(string(&mut r))
            } else {
                None
            };
            let actual = redirect_url(&base, &location).ok();
            if actual != expected {
                eprintln!("{case}: {base:?} + {location:?}: {actual:?} != {expected:?}");
                failures += 1;
            }
        }
        assert_eq!(failures, 0);
        assert_eq!(r.remaining(), 0);
    }
}
