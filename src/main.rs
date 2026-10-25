//! Milestone 1: fetch a page over a raw socket (our own HTTP) and print its text.
//!
//! Usage: cargo run -- https://example.org
mod html;
mod window;

use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Url {
    scheme: String,
    host: String,
    port: u16,
    path: String,
}

impl Url {
    fn parse(input: &str) -> Result<Url> {
        let (scheme, rest) = input
            .split_once("://")
            .ok_or("URL needs a scheme, e.g. https://")?;

        let default_port = match scheme {
            "http" => 80,
            "https" => 443,
            other => return Err(format!("unsupported scheme: {other}").into()),
        };

        // Everything before the first '/' is the host (maybe with :port)
        let (host_part, path) = match rest.find('/') {
            Some(i) => (&rest[..i], rest[i..].to_string()),
            None => (rest, "/".to_string()),
        };

        let (host, port) = match host_part.split_once(':') {
            Some((h, p)) => (h.to_string(), p.parse()?),
            None => (host_part.to_string(), default_port),
        };

        Ok(Url {
            scheme: scheme.to_string(),
            host,
            port,
            path,
        })
    }

    /// Open a socket, send an HTTP request by hand, and return the body.
    fn request(&self) -> Result<(HashMap<String, String>, String)> {
        let tcp = TcpStream::connect((self.host.as_str(), self.port))?;

        // The request is just text. HTTP/1.0 + Connection: close means the
        // server closes the socket when done, so we can read until EOF.
        let request = format!(
            "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\nUser-Agent: toy-browser\r\n\r\n",
            self.path, self.host
        );

        let mut raw = Vec::new();
        if self.scheme == "https" {
            let mut tls = tls_stream(&self.host, tcp)?;
            tls.write_all(request.as_bytes())?;
            read_until_close(&mut tls, &mut raw)?;
        } else {
            let mut tcp = tcp;
            tcp.write_all(request.as_bytes())?;
            tcp.read_to_end(&mut raw)?;
        }

        parse_response(&raw)
    }
}

fn tls_stream(
    host: &str,
    tcp: TcpStream,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string())?;
    let conn = rustls::ClientConnection::new(Arc::new(config), server_name)?;
    Ok(rustls::StreamOwned::new(conn, tcp))
}

/// Lots of servers close TLS without a proper close_notify; treat that as EOF.
fn read_until_close(stream: &mut impl Read, buf: &mut Vec<u8>) -> Result<()> {
    match stream.read_to_end(buf) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn parse_response(raw: &[u8]) -> Result<(HashMap<String, String>, String)> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("malformed response: no end of headers")?;

    let head = String::from_utf8_lossy(&raw[..split]);
    let body = String::from_utf8_lossy(&raw[split + 4..]).into_owned();

    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or("empty response")?;
    eprintln!("[status] {status_line}");

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            // Header names are case-insensitive, so normalize them
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    // We don't speak these yet — bail loudly instead of printing garbage
    if headers.contains_key("transfer-encoding") {
        return Err("chunked transfer encoding not supported yet".into());
    }
    if headers.contains_key("content-encoding") {
        return Err("compressed responses not supported yet".into());
    }

    Ok((headers, body))
}

fn main() -> Result<()> {
    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://example.org/".to_string());
    let url = Url::parse(&input)?;
    let (_headers, body) = url.request()?;
    let tokens = html::tokenizer::tokenize(&body);

    let tree = html::tree::build(tokens);
    let mut text = String::new();

    html::tree::collect_text(&tree, &mut text);
    window::run(text);
    Ok(())
}
