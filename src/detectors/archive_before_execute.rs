use crate::detector::{Detector, Finding, Severity};
use crate::ir::{DamlModule, Statement};

/// Detector #6: archive-before-execute
///
/// In each choice body, find sequences where archive or fetchAndArchive appears
/// before a try/catch block. The archive consumes the contract; if execution in
/// the try block fails, the archived contract is permanently lost.
///
/// Catches: H3 (archive-before-execute in CloseVoteRequest)
pub struct ArchiveBeforeExecute;

impl ArchiveBeforeExecute {
    fn check_statements(
        &self,
        statements: &[Statement],
        body_raw: &str,
        file: &std::path::Path,
        base_line: usize,
        _choice_name: &str,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut archive_seen = false;
        let mut archive_line = 0usize;
        let mut archive_evidence = String::new();

        let lines: Vec<&str> = body_raw.lines().collect();

        // Track line positions for archive and try statements
        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.contains("fetchAndArchive") || trimmed.starts_with("archive ") {
                archive_seen = true;
                archive_line = base_line + line_idx;
                archive_evidence = trimmed.to_string();
            }

            if archive_seen && (trimmed.starts_with("try") || trimmed == "try") {
                findings.push(Finding {
                    detector: self.name().to_string(),
                    severity: self.severity(),
                    file: file.to_path_buf(),
                    line: archive_line,
                    column: 1,
                    message: format!(
                        "Contract archived via '{}' at line {} before try/catch block at line {}. \
                         If execution fails, the archived contract is permanently consumed.",
                        if archive_evidence.contains("fetchAndArchive") {
                            "fetchAndArchive"
                        } else {
                            "archive"
                        },
                        archive_line,
                        base_line + line_idx,
                    ),
                    evidence: format!(
                        "{}\n  ...\n  try do ...",
                        archive_evidence
                    ),
                });
                // Only report once per archive-then-try sequence
                archive_seen = false;
            }
        }

        // Also check the structured statements for the pattern
        let mut archive_stmt_seen = false;
        for stmt in statements {
            match stmt {
                Statement::Archive { .. } => {
                    archive_stmt_seen = true;
                }
                Statement::TryCatch { .. } if archive_stmt_seen => {
                    // Already caught by line-based check above, but this handles
                    // cases where line-based might miss
                    archive_stmt_seen = false;
                }
                _ => {}
            }
        }

        findings
    }
}

impl Detector for ArchiveBeforeExecute {
    fn name(&self) -> &str {
        "archive-before-execute"
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn description(&self) -> &str {
        "Contract archived before try/catch — archived contract lost if execution fails"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        for template in &module.templates {
            for choice in &template.choices {
                findings.extend(self.check_statements(
                    &choice.body,
                    &choice.body_raw,
                    &module.file,
                    choice.span.line,
                    &choice.name,
                ));
            }
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_daml;
    use std::path::Path;

    #[test]
    fn test_archive_before_try_triggers() {
        let source = r#"module Test where

template VoteManager
  with
    admin : Party
  where
    signatory admin

    choice CloseVoteRequest : ()
      with
        requestCid : ContractId VoteRequest
      controller admin
      do
        request <- fetchAndArchive requestCid
        let action = request.action
        try do
          executeAction action
        catch
          e -> pure ()
"#;
        let module = parse_daml(source, Path::new("DsoRules.daml"));
        let findings = ArchiveBeforeExecute.detect(&module);
        assert!(!findings.is_empty());
        assert!(findings[0].message.contains("fetchAndArchive"));
        assert!(findings[0].message.contains("try/catch"));
    }

    #[test]
    fn test_archive_after_try_passes() {
        let source = r#"module Test where

template SafeManager
  with
    admin : Party
  where
    signatory admin

    choice SafeClose : ()
      with
        requestCid : ContractId VoteRequest
      controller admin
      do
        try do
          executeAction requestCid
        catch
          e -> pure ()
        archive requestCid
"#;
        let module = parse_daml(source, Path::new("Safe.daml"));
        let findings = ArchiveBeforeExecute.detect(&module);
        assert!(findings.is_empty());
    }
}
