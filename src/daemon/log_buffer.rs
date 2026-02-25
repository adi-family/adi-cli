use std::collections::HashMap;
use std::sync::RwLock;

const DEFAULT_MAX_LINES: usize = 10_000;

/// Per-service ring buffer for captured stdout/stderr lines.
pub struct LogBuffer {
    max_lines: usize,
    logs: RwLock<HashMap<String, Vec<String>>>,
}

impl LogBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            logs: RwLock::new(HashMap::new()),
        }
    }

    /// Append a line for the given service, trimming oldest if over capacity.
    pub fn push(&self, service: &str, line: String) {
        let mut logs = self.logs.write().expect("LogBuffer lock poisoned");
        let entries = logs.entry(service.to_string()).or_default();
        entries.push(line);
        if entries.len() > self.max_lines {
            let excess = entries.len() - self.max_lines;
            entries.drain(..excess);
        }
    }

    /// Return the last `n` lines for a service (or all if `n` exceeds stored count).
    pub fn tail(&self, service: &str, n: usize) -> Vec<String> {
        let logs = self.logs.read().expect("LogBuffer lock poisoned");
        let Some(entries) = logs.get(service) else {
            return Vec::new();
        };
        let start = entries.len().saturating_sub(n);
        entries[start..].to_vec()
    }

    /// Remove all logs for a service.
    pub fn clear(&self, service: &str) {
        let mut logs = self.logs.write().expect("LogBuffer lock poisoned");
        logs.remove(service);
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LINES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_returns_last_n_lines() {
        let buf = LogBuffer::new(100);
        for i in 0..10 {
            buf.push("svc", format!("line {i}"));
        }
        let lines = buf.tail("svc", 3);
        assert_eq!(lines, vec!["line 7", "line 8", "line 9"]);
    }

    #[test]
    fn tail_unknown_service_returns_empty() {
        let buf = LogBuffer::default();
        assert!(buf.tail("unknown", 10).is_empty());
    }

    #[test]
    fn ring_buffer_evicts_oldest() {
        let buf = LogBuffer::new(5);
        for i in 0..10 {
            buf.push("svc", format!("line {i}"));
        }
        let lines = buf.tail("svc", 100);
        assert_eq!(lines, vec!["line 5", "line 6", "line 7", "line 8", "line 9"]);
    }

    #[test]
    fn clear_removes_service_logs() {
        let buf = LogBuffer::default();
        buf.push("svc", "hello".into());
        buf.clear("svc");
        assert!(buf.tail("svc", 10).is_empty());
    }
}
