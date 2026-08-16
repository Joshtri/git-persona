use crate::domain::{audit::AuditEntry, ports::AuditSink};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_ENTRIES: usize = 300;

pub(crate) struct JsonlAuditSink {
    path: PathBuf,
    lock: Mutex<()>,
}

impl JsonlAuditSink {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }
}

impl AuditSink for JsonlAuditSink {
    fn append(&self, entry: &AuditEntry) -> Result<(), AppError> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::Internal("audit lock poisoned".into()))?;
        let line = serde_json::to_string(entry).map_err(|e| AppError::Internal(e.to_string()))?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| AppError::Internal(e.to_string()))
    }

    fn read_all(&self) -> Result<Vec<AuditEntry>, AppError> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::Internal("audit lock poisoned".into()))?;
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let content =
            fs::read_to_string(&self.path).map_err(|e| AppError::Internal(e.to_string()))?;
        let mut entries: Vec<AuditEntry> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).map_err(|e| AppError::Internal(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        entries.reverse();
        entries.truncate(MAX_ENTRIES);
        Ok(entries)
    }

    fn purge_before(&self, cutoff: DateTime<Utc>) -> Result<u32, AppError> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::Internal("audit lock poisoned".into()))?;
        if !self.path.exists() {
            return Ok(0);
        }
        let content =
            fs::read_to_string(&self.path).map_err(|e| AppError::Internal(e.to_string()))?;
        let all: Vec<AuditEntry> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        let before = all.len() as u32;
        let kept: Vec<_> = all.into_iter().filter(|e| e.timestamp >= cutoff).collect();
        let removed = before - kept.len() as u32;
        if removed == 0 {
            return Ok(0);
        }
        let mut file = fs::File::create(&self.path).map_err(|e| AppError::Internal(e.to_string()))?;
        for entry in &kept {
            let line = serde_json::to_string(entry).map_err(|e| AppError::Internal(e.to_string()))?;
            writeln!(file, "{line}").map_err(|e| AppError::Internal(e.to_string()))?;
        }
        Ok(removed)
    }
}
