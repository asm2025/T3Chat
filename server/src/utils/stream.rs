use anyhow::Result;
use bytes::Bytes;
use futures::{Stream, StreamExt};

/// Stream that buffers chunks and yields complete lines for SSE parsing.
/// This handles the case where a line split occurs across chunk boundaries.
pub fn sse_stream(
    stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
) -> impl Stream<Item = Result<String>> + Send {
    stream
        .scan(Vec::<u8>::new(), |buffer: &mut Vec<u8>, item: reqwest::Result<Bytes>| {
            let res = match item {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);

                    let mut lines = Vec::new();
                    let mut start = 0;

                    for (i, &b) in buffer.iter().enumerate() {
                        if b == b'\n' {
                            // Convert bytes to string (lossy to handle potential partial UTF-8 sequences at boundaries safely enough for SSE data lines)
                            // Note: A strict UTF-8 decoder would be better but requires more complex state management.
                            // Since SSE data is usually ASCII/UTF-8 and newlines are single bytes, this is generally safe
                            // as long as we don't split a multi-byte char right before a newline (which is rare/impossible in valid UTF-8).
                            let line = String::from_utf8_lossy(&buffer[start..i]).to_string();
                            lines.push(line);
                            start = i + 1;
                        }
                    }

                    if start > 0 {
                        // Remove processed lines from buffer
                        *buffer = buffer.split_off(start);
                    }

                    Some(Ok(lines))
                }
                Err(e) => Some(Err(anyhow::anyhow!(e))),
            };
            futures::future::ready(res)
        })
        .map(|res: Result<Vec<String>>| match res {
            Ok(lines) => futures::stream::iter(lines.into_iter().map(Ok::<String, anyhow::Error>).collect::<Vec<_>>()),
            Err(e) => futures::stream::iter(vec![Err(e)]),
        })
        .flatten()
}
