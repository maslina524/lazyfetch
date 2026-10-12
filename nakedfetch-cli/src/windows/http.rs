use core::{ffi::c_void, ptr};

use alloc::{string::String, vec::Vec};

use crate::{
    url::{Response, Url},
    windows::{
        encoding::wide,
        error::ErrorCode,
        link::{
            WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
            WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
        },
    },
};

const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: u32 = 0;
const WINHTTP_NO_PROXY_NAME: *const u16 = ptr::null();
const WINHTTP_NO_PROXY_BYPASS: *const u16 = ptr::null();
const WINHTTP_NO_REFERER: *const u16 = ptr::null();
const WINHTTP_DEFAULT_ACCEPT_TYPES: *const *const u16 = ptr::null();
const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
const WINHTTP_NO_ADDITIONAL_HEADERS: *const u16 = ptr::null();
const WINHTTP_NO_REQUEST_DATA: *mut c_void = ptr::null_mut();
const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;

pub struct Request {
    url: Url,
}

impl Request {
    pub fn new(url: impl Into<String>) -> Option<Self> {
        let url = Url::new(url.into())?;
        Some(Self { url })
    }

    pub const fn from_url(url: Url) -> Self {
        Self { url }
    }

    pub fn get(self) -> Result<Response, ErrorCode> {
        self.send("GET")
    }

    fn send(self, method: &str) -> Result<Response, ErrorCode> {
        // SAFETY: Parameters are fully correct, return value is checked
        let session = unsafe {
            let header = wide("UserAgent/1.0");
            WinHttpOpen(
                header.as_ptr(),
                WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                WINHTTP_NO_PROXY_NAME,
                WINHTTP_NO_PROXY_BYPASS,
                0,
            )
        };
        if session.is_null() {
            ErrorCode::last().panic();
        }

        let cap = self.url.domain.len() + self.url.tld.len() + 16;
        let mut hostname = String::with_capacity(cap);
        if !self.url.subdomains.is_empty() {
            hostname.push_str(&self.url.subdomains.join("."));
            hostname.push('.');
        }
        hostname.push_str(&self.url.domain);
        hostname.push('.');
        hostname.push_str(&self.url.tld);

        let server = wide(&hostname);
        // SAFETY: Just a WinAPI function, the return value is checked
        let conn = unsafe { WinHttpConnect(session, server.as_ptr(), self.url.port(), 0) };
        if conn.is_null() {
            let err = ErrorCode::last();
            // SAFETY: Completely safe
            unsafe { WinHttpCloseHandle(session) };
            return Err(err);
        }

        let method = wide(method);
        let path = wide(&self.url.path);
        let flags = if self.url.protocol == "https" {
            WINHTTP_FLAG_SECURE
        } else {
            0
        };

        // SAFETY: Parameters are fully correct, return value is checked
        let req = unsafe {
            WinHttpOpenRequest(
                conn,
                method.as_ptr(),
                path.as_ptr(),
                ptr::null(),
                WINHTTP_NO_REFERER,
                WINHTTP_DEFAULT_ACCEPT_TYPES,
                flags,
            )
        };
        if req.is_null() {
            let err = ErrorCode::last();
            // SAFETY: Completely safe
            unsafe {
                WinHttpCloseHandle(conn);
                WinHttpCloseHandle(session);
            }
            return Err(err);
        }

        // SAFETY: Just a WinAPI function, the return value is checked
        let send = unsafe {
            WinHttpSendRequest(
                req,
                WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                WINHTTP_NO_REQUEST_DATA,
                0,
                0,
                0,
            )
        };
        if send == 0 {
            let err = ErrorCode::last();
            // SAFETY: Completely safe
            unsafe {
                WinHttpCloseHandle(req);
                WinHttpCloseHandle(conn);
                WinHttpCloseHandle(session);
            };
            return Err(err);
        }

        // SAFETY: Just a WinAPI function, the return value is checked
        let ret = unsafe { WinHttpReceiveResponse(req, ptr::null_mut()) };
        if ret == 0 {
            let err = ErrorCode::last();
            // SAFETY: Completely safe
            unsafe {
                WinHttpCloseHandle(req);
                WinHttpCloseHandle(conn);
                WinHttpCloseHandle(session);
            };
            return Err(err);
        }

        let mut status_code: u32 = 0;
        let mut size = size_of::<u32>() as u32;
        // SAFETY: Parameters are fully correct, return value is checked
        let query = unsafe {
            WinHttpQueryHeaders(
                req,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                ptr::null(),
                (&raw mut status_code).cast(),
                &raw mut size,
                ptr::null_mut(),
            )
        };
        if query == 0 {
            return Err(ErrorCode::last());
        }

        let mut buf = Vec::with_capacity(1024);
        let mut read = 0;
        loop {
            let mut chunk = [0u8; 1024];
            // SAFETY: Parameters are fully correct, return value is checked
            let ret =
                unsafe { WinHttpReadData(req, chunk.as_mut_ptr().cast(), 1024, &raw mut read) };
            if ret == 0 || read == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..read as usize]);
        }

        // SAFETY: Completely safe
        unsafe {
            WinHttpCloseHandle(req);
            WinHttpCloseHandle(conn);
            WinHttpCloseHandle(session);
        }

        Ok(Response::new(status_code as u16, buf))
    }
}

#[cfg(test)]
mod tests {
    use crate::windows::http::{Request, Url};

    extern crate std;

    #[test]
    fn example_response_test() {
        let url = "http://wttr.in/?format=%c;%C;%x;%h;%t;%f;%H;%L;%w;%l;%m;%M;%p;%P;%e;%u;%D;%S;%z;%s;%d;%T;%Z";
        let response = Request::new(url).unwrap().get().expect("Failed");
        println!("Response code: {}", response.code());

        let string = String::from_utf8(response.into_content()).unwrap();
        assert!(string.split(';').next().is_some());
    }

    #[test]
    fn url_parse_test() {
        let string = "http://wttr.in/";
        let url = Url::new(string);
        std::println!("{url:?}");
    }
}
