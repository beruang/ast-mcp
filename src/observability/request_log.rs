use std::collections::VecDeque;

use crate::shared::types_v5::{RequestLogEntry, RequestLogResult, RequestStatus};

/// Ring buffer for request log entries.
pub struct RequestLog {
    entries: VecDeque<RequestLogEntry>,
    max_entries: usize,
}

impl RequestLog {
    pub fn new(max_entries: usize) -> Self {
        RequestLog { entries: VecDeque::with_capacity(max_entries.min(1024)), max_entries }
    }

    pub fn push(&mut self, entry: RequestLogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn query(
        &self,
        tool: Option<&str>,
        status: Option<&RequestStatus>,
        file_path: Option<&str>,
        limit: usize,
    ) -> RequestLogResult {
        let filtered: Vec<&RequestLogEntry> = self
            .entries
            .iter()
            .rev()
            .filter(|e| {
                if let Some(t) = tool {
                    if e.tool != t {
                        return false;
                    }
                }
                if let Some(s) = status {
                    if std::mem::discriminant(&e.status) != std::mem::discriminant(s) {
                        return false;
                    }
                }
                if let Some(fp) = file_path {
                    match &e.file_path {
                        Some(efp) if efp.contains(fp) => {}
                        _ => return false,
                    }
                }
                true
            })
            .take(limit)
            .collect();

        let total_stored = self.entries.len();
        let returned = filtered.len();

        RequestLogResult {
            entries: filtered.into_iter().cloned().collect(),
            returned,
            total_stored,
        }
    }

    pub fn clear(&mut self, tool: Option<&str>) -> usize {
        if let Some(t) = tool {
            let before = self.entries.len();
            self.entries.retain(|e| e.tool != t);
            before - self.entries.len()
        } else {
            let count = self.entries.len();
            self.entries.clear();
            count
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(tool: &str, status: RequestStatus) -> RequestLogEntry {
        RequestLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            tool: tool.to_string(),
            started_at: "2026-01-01T00:00:00Z".into(),
            completed_at: Some("2026-01-01T00:00:01Z".into()),
            duration_ms: Some(1000),
            status,
            error_code: None,
            error_message: None,
            file_path: Some("src/test.rs".into()),
            result_count: Some(5),
        }
    }

    #[test]
    fn ring_buffer_wraps() {
        let mut log = RequestLog::new(3);
        log.push(make_entry("ast_parse_file", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Ok));
        log.push(make_entry("ast_find_imports", RequestStatus::Error));
        log.push(make_entry("ast_find_functions", RequestStatus::Ok));
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn filter_by_status() {
        let mut log = RequestLog::new(10);
        log.push(make_entry("ast_parse_file", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Timeout));
        log.push(make_entry("ast_find_imports", RequestStatus::Ok));
        let result = log.query(None, Some(&RequestStatus::Timeout), None, 10);
        assert_eq!(result.returned, 1);
        assert_eq!(result.entries[0].tool, "ast_query");
    }

    #[test]
    fn filter_by_tool() {
        let mut log = RequestLog::new(10);
        log.push(make_entry("ast_parse_file", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Error));
        let result = log.query(Some("ast_query"), None, None, 10);
        assert_eq!(result.returned, 2);
    }

    #[test]
    fn clear_with_filter() {
        let mut log = RequestLog::new(10);
        log.push(make_entry("ast_parse_file", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Ok));
        log.push(make_entry("ast_query", RequestStatus::Ok));
        let cleared = log.clear(Some("ast_query"));
        assert_eq!(cleared, 2);
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn clear_all() {
        let mut log = RequestLog::new(10);
        log.push(make_entry("a", RequestStatus::Ok));
        log.push(make_entry("b", RequestStatus::Ok));
        let cleared = log.clear(None);
        assert_eq!(cleared, 2);
        assert_eq!(log.len(), 0);
    }
}
