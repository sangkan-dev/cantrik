//! Read newline-delimited chunks from a reqwest byte stream into a string buffer.

use futures_util::{Stream, StreamExt};

use super::LlmError;

pub async fn for_each_line<S>(
    mut stream: S,
    mut handle_line: impl FnMut(&str) -> Result<(), LlmError>,
) -> Result<(), LlmError>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    let mut buf = String::new();
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| LlmError::Http(e.to_string()))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(i) = buf.find('\n') {
            let line = buf[..i].trim_end_matches('\r').to_string();
            buf.drain(..=i);
            if !line.is_empty() {
                handle_line(&line)?;
            }
        }
    }
    let tail = buf.trim();
    if !tail.is_empty() {
        handle_line(tail)?;
    }
    Ok(())
}
