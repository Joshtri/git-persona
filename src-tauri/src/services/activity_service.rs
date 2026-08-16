use crate::domain::{audit::AuditEntry, ports::AuditSink};
use crate::error::AppError;
use chrono::{Duration, Utc};
use std::sync::Arc;

const AUTO_PURGE_DAYS: i64 = 30;

pub(crate) struct ActivityService {
    sink: Arc<dyn AuditSink>,
}

impl ActivityService {
    pub(crate) fn new(sink: Arc<dyn AuditSink>) -> Self {
        Self { sink }
    }

    pub(crate) fn list(&self) -> Result<Vec<AuditEntry>, AppError> {
        let cutoff = Utc::now() - Duration::days(AUTO_PURGE_DAYS);
        let _ = self.sink.purge_before(cutoff);
        self.sink.read_all()
    }

    /// Purge entries older than `days`. Pass `0` to clear the entire log.
    pub(crate) fn purge(&self, days: u32) -> Result<u32, AppError> {
        let cutoff = if days == 0 {
            Utc::now() + Duration::seconds(1)
        } else {
            Utc::now() - Duration::days(days as i64)
        };
        self.sink.purge_before(cutoff)
    }
}
