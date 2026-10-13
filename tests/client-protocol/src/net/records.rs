//! An independent HTTP chunk/SSE parser retains only complete received records.
#[derive(Default)]
pub(super) struct Records {
    offset: Option<usize>,
    bytes: Vec<u8>,
    line: usize,
    pub values: Vec<Box<[u8]>>,
}
impl Records {
    pub fn feed(&mut self, http: &[u8]) {
        let Some(mut at) = self.offset.or_else(|| super::body(http).map(|body| http.len() - body.len())) else {
            return;
        };
        while let Some(chunks) = http.get(at..) {
            let Some(end) = chunks.windows(2).position(|b| b == b"\r\n") else { break };
            let Ok(text) = std::str::from_utf8(&chunks[..end]) else { break };
            let Ok(length) = usize::from_str_radix(text, 16) else { break };
            let Some(chunk) = chunks.get(end + 2..end + 2 + length) else { break };
            if chunks.len() < end + 4 + length {
                break;
            }
            self.bytes.extend_from_slice(chunk);
            at += end + 4 + length;
        }
        self.offset = Some(at);
        while let Some(end) = self.bytes[self.line..].iter().position(|b| *b == b'\n') {
            let event = &self.bytes[self.line..self.line + end];
            self.line += end + 1;
            if let Some(value) = event.strip_prefix(b"data: ") {
                let decoded = skein_lib::base64::decode(
                    skein_lib::base64::Alphabet::Standard,
                    value.strip_suffix(b"\r").unwrap_or(value),
                    65536,
                )
                .expect("a complete encoded stream record");
                self.values.push(decoded);
            }
        }
    }
}
