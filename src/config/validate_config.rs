use crate::shared::types_v5::RejectedConfigUpdate;

use super::runtime_config::{
    HARD_MAX_FILE_BYTES, HARD_MAX_PARALLELISM, HARD_MAX_REQUEST_LOG_ENTRIES,
};

pub struct RejectedUpdate {
    pub path: String,
    pub reason: String,
}

impl From<RejectedUpdate> for RejectedConfigUpdate {
    fn from(r: RejectedUpdate) -> Self {
        RejectedConfigUpdate { path: r.path, reason: r.reason }
    }
}

pub fn validate_limit_usize(
    target: &mut usize,
    value: usize,
    path: &str,
    rejected: &mut Vec<RejectedUpdate>,
) {
    if value == 0 {
        rejected
            .push(RejectedUpdate { path: path.into(), reason: "value must be positive".into() });
    } else if path == "limits.max_file_bytes" && value > HARD_MAX_FILE_BYTES {
        rejected.push(RejectedUpdate {
            path: path.into(),
            reason: format!("value {} exceeds hard safety ceiling {}", value, HARD_MAX_FILE_BYTES),
        });
    } else {
        *target = value;
    }
}

pub fn validate_timeout(
    target: &mut u64,
    value: u64,
    path: &str,
    rejected: &mut Vec<RejectedUpdate>,
) {
    if value == 0 {
        rejected
            .push(RejectedUpdate { path: path.into(), reason: "timeout must be positive".into() });
    } else {
        *target = value;
    }
}

pub fn validate_parallelism(
    target: &mut usize,
    value: usize,
    path: &str,
    rejected: &mut Vec<RejectedUpdate>,
) {
    if value == 0 {
        rejected.push(RejectedUpdate {
            path: path.into(),
            reason: "max_parallelism must be at least 1".into(),
        });
    } else if value > HARD_MAX_PARALLELISM {
        rejected.push(RejectedUpdate {
            path: path.into(),
            reason: format!(
                "max_parallelism {} exceeds hard ceiling {}",
                value, HARD_MAX_PARALLELISM
            ),
        });
    } else {
        *target = value;
    }
}

pub fn validate_request_log_entries(
    target: &mut usize,
    value: usize,
    path: &str,
    rejected: &mut Vec<RejectedUpdate>,
) {
    if value == 0 {
        rejected.push(RejectedUpdate {
            path: path.into(),
            reason: "request_log_max_entries must be positive".into(),
        });
    } else if value > HARD_MAX_REQUEST_LOG_ENTRIES {
        rejected.push(RejectedUpdate {
            path: path.into(),
            reason: format!(
                "request_log_max_entries {} exceeds hard ceiling {}",
                value, HARD_MAX_REQUEST_LOG_ENTRIES
            ),
        });
    } else {
        *target = value;
    }
}
