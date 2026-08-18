//! A canned-response HTTP server, so the client tests need no mock-HTTP crate.

pub mod fixtures;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// What the server saw for one request.
#[derive(Debug, Clone)]
pub struct ReceivedRequest {
    pub authorization: Option<String>,
    pub body: String,
}

pub struct MockServer {
    url: String,
    received: Arc<Mutex<Vec<ReceivedRequest>>>,
}

impl MockServer {
    /// Serve one canned `(status, body)` reply per element, in order, then stop.
    pub fn spawn(replies: Vec<(u16, String)>) -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let received = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&received);

        std::thread::spawn(move || {
            for (status, body) in replies {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                handle(stream, status, &body, &sink);
            }
        });

        MockServer {
            url: format!("http://{addr}"),
            received,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn requests(&self) -> Vec<ReceivedRequest> {
        self.received.lock().expect("lock").clone()
    }
}

// Reads one request, then replies. The request is recorded (by the caller,
// via `sink`) strictly before the reply is written, so a client can never
// observe the reply before `requests()` reflects the request that earned it.
fn handle(mut stream: TcpStream, status: u16, body: &str, sink: &Mutex<Vec<ReceivedRequest>>) {
    let Some((authorization, buf)) = read_request(&mut stream) else {
        return;
    };
    sink.lock().expect("lock").push(ReceivedRequest {
        authorization,
        body: String::from_utf8_lossy(&buf).into_owned(),
    });

    let reason = if status == 200 { "OK" } else { "Error" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    // A write failure must not un-record a request we already fully read.
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn read_request(stream: &mut TcpStream) -> Option<(Option<String>, Vec<u8>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut authorization = None;
    let mut content_length = 0usize;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if name == "authorization" {
                authorization = Some(value);
            } else if name == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
        }
    }

    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf).ok()?;
    Some((authorization, buf))
}
