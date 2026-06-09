use std::sync::Mutex;
use std::time::Instant;

use crate::shared::types_v5::{RequestLogEntry, RequestStatus};

use super::request_log::RequestLog;

pub struct RequestTracker {
    log: Mutex<RequestLog>,
}

impl RequestTracker {
    pub fn new(max_entries: usize) -> Self {
        RequestTracker { log: Mutex::new(RequestLog::new(max_entries)) }
    }

    pub fn track(
        &self,
        tool: &str,
        file_path: Option<&str>,
        f: impl FnOnce() -> (RequestStatus, Option<String>, Option<String>, Option<usize>),
    ) {
        let id = uuid::Uuid::new_v4().to_string();
        let started_at = chrono_now();
        let start = Instant::now();

        let (status, error_code, error_message, result_count) = f();

        let duration_ms = Some(start.elapsed().as_millis() as u64);
        let completed_at = Some(chrono_now());

        let entry = RequestLogEntry {
            id,
            tool: tool.to_string(),
            started_at,
            completed_at,
            duration_ms,
            status,
            error_code,
            error_message,
            file_path: file_path.map(|s| s.to_string()),
            result_count,
        };

        self.log.lock().unwrap().push(entry);
    }

    pub fn query(
        &self,
        tool: Option<&str>,
        status: Option<&RequestStatus>,
        file_path: Option<&str>,
        limit: usize,
    ) -> crate::shared::types_v5::RequestLogResult {
        self.log.lock().unwrap().query(tool, status, file_path, limit)
    }

    pub fn clear(&self, tool: Option<&str>) -> usize {
        self.log.lock().unwrap().clear(tool)
    }

    pub fn len(&self) -> usize {
        self.log.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn chrono_now() -> String {
    // Simple ISO 8601 without pulling in chrono crate
    let now =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    // Format as ISO 8601
    let seconds = (secs % 60) as u8;
    let minutes = ((secs / 60) % 60) as u8;
    let hours = ((secs / 3600) % 24) as u8;
    // Days since epoch — keep it simple
    format!("{}s-{:02}:{:02}:{:02}Z", secs / 86400, hours, minutes, seconds)
}
