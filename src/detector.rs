use crate::ir::DamlModule;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub detector: String,
    pub severity: Severity,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub evidence: String,
}

pub trait Detector: Send + Sync {
    fn name(&self) -> &str;
    fn severity(&self) -> Severity;
    fn description(&self) -> &str;
    fn detect(&self, module: &DamlModule) -> Vec<Finding>;
}

use crate::detectors::archive_before_execute::ArchiveBeforeExecute;
use crate::detectors::ensure_decimal::MissingEnsureDecimal;
use crate::detectors::head_of_list::HeadOfListQuery;
use crate::detectors::positive_amount::MissingPositiveAmount;
use crate::detectors::unbounded_fields::UnboundedFields;
use crate::detectors::unguarded_division::UnguardedDivision;

pub fn all_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(MissingEnsureDecimal),
        Box::new(UnguardedDivision),
        Box::new(HeadOfListQuery),
        Box::new(UnboundedFields),
        Box::new(MissingPositiveAmount),
        Box::new(ArchiveBeforeExecute),
    ]
}
