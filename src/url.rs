use alloc::{borrow::ToOwned, string::String, vec::Vec};

use crate::format;

#[derive(Debug)]
pub struct Url {
    pub protocol: String,
    pub subdomains: Vec<String>,
    pub domain: String,
    pub tld: String,
    pub port: Option<u16>,
    pub path: String,
    // no query
}

impl Url {
    pub fn new(url: impl Into<String>) -> Option<Self> {
        let mut url = url.into();

        // Protocol
        let proto_sep = url.find("://")?;
        let protocol = url[..proto_sep].to_owned();
        url = url[proto_sep + 3..].to_owned();

        // Path
        let slash_sep = url.find('/').unwrap_or(url.len());
        let path = if slash_sep == url.len() {
            String::from("/")
        } else {
            url[slash_sep..].to_owned()
        };
        let mut base = url[..slash_sep].to_owned();

        // Port
        let port = if let Some(colon_sep) = base.rfind(':') {
            let port_string = &base[colon_sep + 1..];
            let port = port_string.parse::<u16>().ok()?;
            base = base[..colon_sep].to_owned();
            Some(port)
        } else {
            None
        };

        let mut parts = base
            .split('.')
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>();

        // Tld
        let tld = parts.pop()?;
        // Domain
        let domain = parts.pop()?;
        // Subdomains
        let subdomains = parts;

        Some(Self {
            protocol,
            subdomains,
            domain,
            tld,
            port,
            path,
        })
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(match self.protocol.as_str() {
            "http" => 80,
            "https" => 443,
            _ => 0,
        })
    }
}

impl core::fmt::Display for Url {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // protocol
        let mut ret = format!("{}://", self.protocol);

        // subdomains
        if !self.subdomains.is_empty() {
            ret.push_str(&format!("{}.", self.subdomains.join(".")));
        }

        // domain.tld
        ret.push_str(&format!("{}.{}", self.domain, self.tld));

        // port
        if let Some(port) = self.port {
            ret.push_str(&format!(":{}", port));
        }

        // path
        ret.push_str(&self.path);

        write!(f, "{ret}")
    }
}

#[derive(Debug)]
pub struct Response {
    code: u16,
    content: Vec<u8>,
}

impl Response {
    pub const fn new(code: u16, content: Vec<u8>) -> Self {
        Self { code, content }
    }

    pub fn from_raw(data: &[u8]) -> Self {
        let header_end = data
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap_or(data.len());

        let headers = &data[..header_end];
        let content = if header_end + 4 <= data.len() {
            data[header_end + 4..].to_vec()
        } else {
            Vec::new()
        };

        let headers_str = String::from_utf8_lossy(headers);
        let status_line = headers_str.lines().next().unwrap_or("");

        let mut parts = status_line.split_whitespace();
        let _version = parts.next();
        let code_str = parts.next();
        let _reason = parts.next();

        let code = code_str.and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);

        Self { code, content }
    }

    pub const fn code(&self) -> u16 {
        self.code
    }

    pub const fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    pub const fn content(&self) -> &Vec<u8> {
        &self.content
    }

    pub fn into_content(self) -> Vec<u8> {
        self.content
    }

    pub fn as_text(&self) -> Result<String, alloc::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }
}
