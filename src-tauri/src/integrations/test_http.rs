use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Keeps connections open with empty JSON bodies and counts accepted connections.
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
