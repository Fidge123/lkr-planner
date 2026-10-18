use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Answers every request with an empty JSON body and keeps the connection open,
/// counting accepted connections so a test can tell whether a client reused one.
pub(crate) async fn spawn_keep_alive_server() -> (String, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let addr = listener.local_addr().expect("test server addr");
    let connections = Arc::new(AtomicUsize::new(0));
    let accepted = connections.clone();

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            accepted.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut pending = Vec::new();
                let mut chunk = [0u8; 1024];
                loop {
                    while let Some(length) = complete_request_length(&pending) {
                        pending.drain(..length);
                        let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
                        if stream.write_all(response.as_bytes()).await.is_err() {
                            return;
                        }
                    }
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(read) => pending.extend_from_slice(&chunk[..read]),
                    }
                }
            });
        }
    });

    (format!("http://{addr}"), connections)
}

fn complete_request_length(buffer: &[u8]) -> Option<usize> {
    let head_end = buffer.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    let head = String::from_utf8_lossy(&buffer[..head_end]);
    let body_length: usize = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())?
        })
        .unwrap_or(0);

    (buffer.len() >= head_end + body_length).then_some(head_end + body_length)
}

/// Routes are matched by method + path prefix, and every request is recorded
/// in arrival order.
pub(crate) struct TestServer {
    pub(crate) base_url: String,
    received: Arc<Mutex<Vec<(String, String)>>>,
}

impl TestServer {
    pub(crate) async fn spawn(routes: Vec<(&'static str, &'static str, u16)>) -> Self {
        Self::spawn_with_bodies(
            routes
                .into_iter()
                .map(|(method, prefix, status)| (method, prefix, status, ""))
                .collect(),
        )
        .await
    }

    pub(crate) async fn spawn_with_bodies(
        routes: Vec<(&'static str, &'static str, u16, &'static str)>,
    ) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("test server addr");
        let received: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let recorded = received.clone();

        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let routes = routes.clone();
                let recorded = recorded.clone();
                tokio::spawn(async move {
                    let Some((method, path)) = read_request(&mut stream).await else {
                        return;
                    };
                    recorded
                        .lock()
                        .unwrap()
                        .push((method.clone(), path.clone()));
                    let (status, body) = routes
                        .iter()
                        .find(|(m, prefix, _, _)| *m == method && path.starts_with(prefix))
                        .map(|(_, _, status, body)| (*status, *body))
                        .unwrap_or((404, ""));
                    let response = format!(
                        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.shutdown().await;
                });
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            received,
        }
    }

    pub(crate) fn requests(&self) -> Vec<(String, String)> {
        self.received.lock().unwrap().clone()
    }
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Option<(String, String)> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let head_end = loop {
        let read = stream.read(&mut chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
    };

    let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
    let content_length: usize = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())?
        })
        .unwrap_or(0);

    // Drain the body so the client can finish writing before we respond.
    let mut body_read = buffer.len() - head_end;
    while body_read < content_length {
        let read = stream.read(&mut chunk).await.ok()?;
        if read == 0 {
            break;
        }
        body_read += read;
    }

    let mut request_line = head.lines().next()?.split_whitespace();
    let method = request_line.next()?.to_string();
    let path = request_line.next()?.to_string();
    Some((method, path))
}
