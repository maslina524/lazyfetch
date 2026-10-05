use core::{
    ffi::c_int, 
    ptr
};

use alloc::{
    ffi::CString, 
    string::{String, ToString}, 
    vec::Vec,
};

use crate::{
    abort,
    unix::{
        libc::{
            AddrInfo, close, connect, freeaddrinfo, 
            getaddrinfo, recv, send, socket
        },
        error::ErrorCode
    },
    url::{Response, Url},
    format
};

const BUF_SIZE: usize = 1024;

const AF_UNSPEC: c_int = 0;
const SOCK_STREAM: c_int = 1;
const IPPROTO_TCP: c_int = 6;

pub struct Request {
    url: Url,
}

impl Request {
    pub fn new(url: impl Into<String>) -> Option<Self> {
        let url = Url::new(url.into())?;
        Some(
            Self { url }
        )
    }

    pub const fn from_url(url: Url) -> Self {
        Self { url }
    }

    pub fn get(self) -> Result<Response, ErrorCode> {
        self.send("GET")
    }

    fn send(self, method: &str) -> Result<Response, ErrorCode> {
        // crate::println!("Url: {:#?}", self.url);
        let full_domain = if self.url.subdomains.is_empty() {
            format!("{}.{}", self.url.domain, self.url.tld)
        } else {
            format!(
                "{}.{}.{}",
                self.url.subdomains.join("."),
                self.url.domain,
                self.url.tld
            )
        };
        // crate::println!("Full domain: {}", full_domain);

        let c_hostname = CString::new(full_domain.clone()).expect("invalid hostname");
        let c_port = CString::new(self.url.port().to_string()).expect("invalid port");

        let hints = AddrInfo {
            ai_family: AF_UNSPEC,
            ai_socktype: SOCK_STREAM,
            ai_protocol: IPPROTO_TCP,
            ..Default::default()
        };

        let mut result: *mut AddrInfo = ptr::null_mut();
        let ret = getaddrinfo(
            c_hostname.as_ptr(),
            c_port.as_ptr(),
            &raw const hints,
            &raw mut result,
        );
        if ret != 0 {
            let err = ErrorCode::new(ret.abs());
            return Err(err);
        }

        let mut sockfd: c_int = -1;
        let mut rp = result;
        while !rp.is_null() {
            // SAFETY: Libc always returns a valid pointer
            let ai = unsafe { &*rp };
            // crate::println!(
            //     "ai: family={} socktype={} proto={} addrlen={} addr={:p} next={:p}",
            //     ai.ai_family, ai.ai_socktype, ai.ai_protocol,
            //     ai.ai_addrlen, ai.ai_addr, ai.ai_next
            // );

            sockfd = socket(ai.ai_family, ai.ai_socktype, ai.ai_protocol);
            if sockfd == -1 {
                rp = ai.ai_next;
                continue;
            }
            if connect(sockfd, ai.ai_addr, ai.ai_addrlen) == 0 {
                break;
            }
            close(sockfd);
            sockfd = -1;
            rp = ai.ai_next;
        }

        freeaddrinfo(result);

        if sockfd == -1 {
            abort!("Failed to connect to host");
        }

        let cap = 23 + 17 + 1 + method.len() + self.url.path.len() + full_domain.len() + 5;
        let mut req = String::with_capacity(cap);
        req.push_str(method);
        req.push(' ');
        req.push_str(&self.url.path);
        req.push_str(" HTTP/1.1\r\nHost: ");
        req.push_str(&full_domain);
        if let Some(port) = self.url.port { 
            use core::fmt::Write;
            let _ = write!(req, ":{port}");
        }
        req.push_str("\r\nConnection: close\r\n\r\n");

        let req_bytes = req.as_bytes();

        let sent = send(sockfd, req_bytes.as_ptr().cast(), req_bytes.len(), 0);
        if sent == -1 {
            close(sockfd);
            abort!("Send failed");
        }

        let mut response_data = Vec::with_capacity(BUF_SIZE);
        let mut buffer = [0u8; BUF_SIZE];
        loop {
            let n = recv(sockfd, buffer.as_mut_ptr().cast(), buffer.len(), 0);
            if n < 0 {
                close(sockfd);
                abort!("Recv failed");
            }
            if n == 0 {
                break;
            }
            response_data.extend_from_slice(&buffer[..n as usize]);
        }

        close(sockfd);

        Ok(Response::from_raw(&response_data))
    }
}