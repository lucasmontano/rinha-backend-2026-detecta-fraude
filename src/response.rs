fn build_response(score_str: &str, approved: bool) -> Vec<u8> {
    let body = format!(
        r#"{{"approved":{},"fraud_score":{}}}"#,
        if approved { "true" } else { "false" },
        score_str,
    );
    let header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len());
    let mut v = Vec::with_capacity(header.len() + body.len());
    v.extend_from_slice(header.as_bytes());
    v.extend_from_slice(body.as_bytes());
    v
}

pub struct Responses {
    pub by_count: [Vec<u8>; 6],
    pub ready: Vec<u8>,
    pub fallback: Vec<u8>,
}

impl Responses {
    pub fn new() -> Self {
        const SCORES: [&str; 6] = ["0", "0.2", "0.4", "0.6", "0.8", "1"];
        let mut by_count: [Vec<u8>; 6] = Default::default();
        for i in 0..6 {
            let approved = i < 3;
            by_count[i] = build_response(SCORES[i], approved);
        }
        let ready = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".to_vec();
        let fallback = build_response("0.0", true);
        Responses {
            by_count,
            ready,
            fallback,
        }
    }

    #[inline]
    pub fn for_count(&self, fraud_count: u8) -> &[u8] {
        let i = (fraud_count.min(5)) as usize;
        &self.by_count[i]
    }
}

impl Default for Responses {
    fn default() -> Self {
        Self::new()
    }
}
