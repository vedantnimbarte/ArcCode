//! J3 inbound webhook receiver — a minimal, dependency-free HTTP/1.1
//! endpoint that turns `POST /goals` into an intake [`Goal`].
//!
//! Built on `std::net::TcpListener` so it pulls in no web framework and is
//! fully testable over a loopback socket. The daemon binds this on a
//! configured port; each accepted goal is handed to the caller's callback
//! (which enqueues it for the same auto/notify/gate path every other
//! channel uses).
//!
//! Scope: this is the local-HTTP intake adapter from the J3 table. Slack
//! and email adapters are thin transforms over the same
//! [`crate::intake::normalize`] pipeline once their transport delivers a
//! body; the HTTP receiver is the one that needs a socket, and it's here.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::intake::{normalize, Channel, Goal};

/// Split the body (everything after the first blank line) out of a raw
/// HTTP request. Returns `None` when there's no body section.
pub fn parse_http_body(raw: &str) -> Option<&str> {
    // Headers and body are separated by CRLFCRLF (tolerate bare LFLF).
    if let Some(idx) = raw.find("\r\n\r\n") {
        return Some(&raw[idx + 4..]);
    }
    if let Some(idx) = raw.find("\n\n") {
        return Some(&raw[idx + 2..]);
    }
    None
}

/// Extract `(text, author)` from a JSON body of the shape
/// `{"goal": "...", "author": "..."}` (also accepts `"text"` for `goal`).
pub fn extract_goal_fields(body: &str) -> Option<(String, Option<String>)> {
    let v: serde_json::Value = serde_json::from_str(body.trim()).ok()?;
    let text = v
        .get("goal")
        .or_else(|| v.get("text"))
        .and_then(|t| t.as_str())?
        .to_string();
    if text.trim().is_empty() {
        return None;
    }
    let author = v
        .get("author")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());
    Some((text, author))
}

/// Read one HTTP request off `stream`, parse a goal from its body, write a
/// minimal response, and return the normalised [`Goal`] (or `None` on a
/// malformed/empty request, after replying `400`).
pub fn handle_connection(
    stream: &mut TcpStream,
    trusted_authors: &[String],
) -> std::io::Result<Option<Goal>> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let raw = String::from_utf8_lossy(&buf[..n]);

    let goal = parse_http_body(&raw)
        .and_then(extract_goal_fields)
        .and_then(|(text, author)| {
            normalize(Channel::Webhook, &text, author.as_deref(), None, trusted_authors)
        });

    let response = if goal.is_some() {
        "HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\naccepted"
    } else {
        "HTTP/1.1 400 Bad Request\r\nContent-Length: 7\r\n\r\nbad req"
    };
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(goal)
}

/// Bind `addr` and serve inbound goals. Convenience wrapper over
/// [`serve_listener`]. Blocking; the daemon runs it on a dedicated thread.
pub fn serve<F>(
    addr: &str,
    trusted_authors: &[String],
    max_requests: usize,
    on_goal: F,
) -> std::io::Result<()>
where
    F: FnMut(Goal),
{
    let listener = TcpListener::bind(addr)?;
    serve_listener(listener, trusted_authors, max_requests, on_goal)
}

/// Serve inbound goals on an already-bound `listener`. Handles
/// `max_requests` connections (0 = serve forever); each parsed goal is
/// passed to `on_goal`. Taking a pre-bound listener lets callers (and
/// tests) learn the port before any client connects, avoiding a
/// bind/connect race.
pub fn serve_listener<F>(
    listener: TcpListener,
    trusted_authors: &[String],
    max_requests: usize,
    mut on_goal: F,
) -> std::io::Result<()>
where
    F: FnMut(Goal),
{
    let mut handled = 0usize;
    for incoming in listener.incoming() {
        let mut stream = incoming?;
        if let Ok(Some(goal)) = handle_connection(&mut stream, trusted_authors) {
            on_goal(goal);
        }
        handled += 1;
        if max_requests != 0 && handled >= max_requests {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intake::TrustLevel;
    use std::net::TcpListener as StdListener;

    #[test]
    fn parse_http_body_handles_crlf_and_lf() {
        assert_eq!(
            parse_http_body("POST /goals HTTP/1.1\r\nHost: x\r\n\r\n{\"goal\":\"hi\"}"),
            Some("{\"goal\":\"hi\"}")
        );
        assert_eq!(
            parse_http_body("POST /goals HTTP/1.1\nHost: x\n\nbody"),
            Some("body")
        );
        assert_eq!(parse_http_body("no body here"), None);
    }

    #[test]
    fn extract_goal_fields_reads_goal_and_author() {
        let (text, author) =
            extract_goal_fields(r#"{"goal":"add dark mode","author":"vedant"}"#).unwrap();
        assert_eq!(text, "add dark mode");
        assert_eq!(author.as_deref(), Some("vedant"));
    }

    #[test]
    fn extract_goal_fields_accepts_text_alias_and_rejects_empty() {
        assert_eq!(
            extract_goal_fields(r#"{"text":"fix it"}"#).unwrap().0,
            "fix it"
        );
        assert!(extract_goal_fields(r#"{"goal":"   "}"#).is_none());
        assert!(extract_goal_fields("not json").is_none());
    }

    #[test]
    fn handle_connection_parses_goal_over_loopback() {
        // Bind an ephemeral port; a client thread POSTs a goal; the
        // server-side handle_connection returns the parsed Goal.
        let listener = StdListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let client = std::thread::spawn(move || {
            let mut s = TcpStream::connect(addr).unwrap();
            let body = r#"{"goal":"add a flag","author":"vedant"}"#;
            let req = format!(
                "POST /goals HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            s.write_all(req.as_bytes()).unwrap();
            s.flush().unwrap();
            // Read the response so the server's write doesn't race teardown.
            let mut resp = String::new();
            let _ = s.read_to_string(&mut resp);
            resp
        });

        let (mut stream, _) = listener.accept().unwrap();
        let goal = handle_connection(&mut stream, &["vedant".to_string()])
            .unwrap()
            .expect("goal parsed");
        assert_eq!(goal.text, "add a flag");
        assert_eq!(goal.source, Channel::Webhook);
        assert_eq!(goal.trust_level, TrustLevel::Trusted);
        // Close the server side so the client's read-to-EOF returns
        // (otherwise join() below deadlocks).
        drop(stream);

        let resp = client.join().unwrap();
        assert!(resp.contains("200 OK"));
    }

    #[test]
    fn serve_listener_handles_bounded_request_count() {
        // Bind first (no rebind race), learn the port, then serve on a
        // thread while a client POSTs one goal.
        let listener = StdListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server = std::thread::spawn(move || {
            let mut goals = Vec::new();
            serve_listener(listener, &[], 1, |g| goals.push(g)).unwrap();
            goals
        });

        let mut s = TcpStream::connect(addr).unwrap();
        let body = r#"{"goal":"do the thing"}"#;
        let req = format!(
            "POST /goals HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        s.write_all(req.as_bytes()).unwrap();
        s.flush().unwrap();
        let mut resp = String::new();
        let _ = s.read_to_string(&mut resp);

        let goals = server.join().unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].text, "do the thing");
    }
}
