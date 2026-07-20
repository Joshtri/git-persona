use crate::domain::{audit::AuditEntry, ports::AuditSink};
use crate::error::AppError;
use std::sync::Arc;

pub(crate) struct ActivityService {
    sink: Arc<dyn AuditSink>,
}

impl ActivityService {
    pub(crate) fn new(sink: Arc<dyn AuditSink>) -> Self {
        Self { sink }
    }

    pub(crate) fn list(&self) -> Result<Vec<AuditEntry>, AppError> {
        self.sink.read_all()
    }
}
