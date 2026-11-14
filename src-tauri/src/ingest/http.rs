//! Minimal HTTP/1.1 for the loopback event server: one request per connection, then close.
//! Hand-rolled instead of a server crate so a held `PermissionRequest` keeps its raw socket (AV-21).

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::server::MAX_BODY;

const MAX_HEAD: usize = 16 * 1024;
pub const READ_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Default)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub declared_len: Option<usize>,
    /// Empty when the declared length is over `MAX_BODY`; the handler answers 413 from `declared_len`.
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }
}

/// Reads one request; `Err(status)` is the status to answer with.
pub fn read_request(stream: impl Read) -> Result<HttpRequest, u16> {
    let mut r = BufReader::new(stream);
    let mut head_len = 0usize;
    let mut line = |r: &mut BufReader<_>| -> Result<String, u16> {
        let mut buf = Vec::new();
        let n = r.by_ref().take((MAX_HEAD - head_len + 1) as u64).read_until(b'\n', &mut buf).map_err(|_| 400u16)?;
        head_len += n;
        if n == 0 || head_len > MAX_HEAD || !buf.ends_with(b"\n") {
            return Err(if head_len > MAX_HEAD { 431 } else { 400 });
        }
        Ok(String::from_utf8_lossy(&buf).trim_end_matches(['\r', '\n']).to_string())
    };
    let start = line(&mut r)?;
    let mut parts = start.split(' ');
    let (Some(method), Some(url), Some(_version)) = (parts.next(), parts.next(), parts.next()) else { return Err(400) };
    let mut req = HttpRequest { method: method.to_string(), url: url.to_string(), ..Default::default() };
    loop {
        let l = line(&mut r)?;
        if l.is_empty() {
            break;
        }
        let Some((k, v)) = l.split_once(':') else { return Err(400) };
        req.headers.push((k.trim().to_string(), v.trim().to_string()));
    }
    let chunked = req.header("Transfer-Encoding").is_some_and(|t| t.to_ascii_lowercase().contains("chunked"));
    if chunked {
        req.body = read_chunked(&mut r)?;
        return Ok(req);
    }
    let len = match req.header("Content-Length") {
        Some(v) => v.parse::<usize>().map_err(|_| 400u16)?,
        None => 0,
    };
    req.declared_len = Some(len);
    if len <= MAX_BODY {
        req.body = vec![0; len];
        r.read_exact(&mut req.body).map_err(|_| 400u16)?;
    }
    Ok(req)
}

fn read_chunked(r: &mut impl BufRead) -> Result<Vec<u8>, u16> {
    let mut body = Vec::new();
    loop {
        let mut size = String::new();
        r.by_ref().take(64).read_line(&mut size).map_err(|_| 400u16)?;
        let size = usize::from_str_radix(size.trim().split(';').next().unwrap_or(""), 16).map_err(|_| 400u16)?;
        if size == 0 {
            // Trailers, then the final blank line.
            loop {
                let mut t = String::new();
                if r.by_ref().take(MAX_HEAD as u64).read_line(&mut t).map_err(|_| 400u16)? == 0 || t.trim().is_empty() {
                    return Ok(body);
                }
            }
        }
        if body.len() + size > MAX_BODY {
            return Err(413);
        }
        let at = body.len();
        body.resize(at + size, 0);
        r.read_exact(&mut body[at..]).map_err(|_| 400u16)?;
        let mut crlf = [0u8; 2];
        r.read_exact(&mut crlf).map_err(|_| 400u16)?;
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Status",
    }
}

pub fn respond(mut w: impl Write, status: u16, json: Option<&str>) -> io::Result<()> {
    let mut out = format!("HTTP/1.1 {status} {}\r\nConnection: close\r\n", reason(status));
    match json {
        Some(b) => out.push_str(&format!("Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{b}", b.len())),
        None if status == 204 => out.push_str("\r\n"),
        None => out.push_str("Content-Length: 0\r\n\r\n"),
    }
    w.write_all(out.as_bytes())?;
    w.flush()
}

/// True once the peer has closed or reset the connection. Never blocks.
pub fn peer_closed(stream: &TcpStream) -> bool {
    if stream.set_nonblocking(true).is_err() {
        return true;
    }
    let mut b = [0u8; 1];
    let gone = match stream.peek(&mut b) {
        Ok(0) => true,
        Ok(_) => false,
        Err(e) => e.kind() != io::ErrorKind::WouldBlock,
    };
    let _ = stream.set_nonblocking(false);
    gone
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Result<HttpRequest, u16> {
        read_request(raw.as_bytes())
    }

    #[test]
    fn reads_content_length_body_and_headers() {
        let r = parse("POST /hook/Stop HTTP/1.1\r\nX-PeepThis-Token: t\r\ncontent-length: 2\r\n\r\n{}").unwrap();
        assert_eq!((r.method.as_str(), r.url.as_str()), ("POST", "/hook/Stop"));
        assert_eq!(r.header("x-peepthis-token"), Some("t"));
        assert_eq!((r.declared_len, r.body.as_slice()), (Some(2), b"{}".as_slice()));
    }

    #[test]
    fn reads_chunked_body() {
        let r = parse("POST /x HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{\"\r\n4\r\na\":1\r\n0\r\n\r\n").unwrap();
        assert_eq!(r.body, br#"{"a":1"#);
    }

    #[test]
    fn oversized_declared_body_is_not_read() {
        let r = parse(&format!("POST /x HTTP/1.1\r\nContent-Length: {}\r\n\r\n", MAX_BODY + 1)).unwrap();
        assert_eq!(r.declared_len, Some(MAX_BODY + 1));
        assert!(r.body.is_empty());
    }

    #[test]
    fn malformed_requests_are_rejected() {
        assert_eq!(parse("garbage\r\n\r\n").unwrap_err(), 400);
        assert_eq!(parse("POST /x HTTP/1.1\r\nno-colon\r\n\r\n").unwrap_err(), 400);
        assert_eq!(parse("POST /x HTTP/1.1\r\nContent-Length: 5\r\n\r\n{}").unwrap_err(), 400);
        assert_eq!(parse(&format!("POST /x HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(MAX_HEAD))).unwrap_err(), 431);
    }

    #[test]
    fn responses_are_well_formed() {
        let mut out = Vec::new();
        respond(&mut out, 204, None).unwrap();
        assert_eq!(out, b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
        let mut out = Vec::new();
        respond(&mut out, 200, Some("{}")).unwrap();
        assert!(String::from_utf8(out).unwrap().ends_with("Content-Length: 2\r\n\r\n{}"));
    }
}
