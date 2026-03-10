use crate::detector::{Detector, Finding, Severity};
use crate::ir::DamlModule;

/// Detector #3: head-of-list-query
///
/// Find pattern matches of the form (x, y) :: _ or [(x)] on the result of
/// query, queryFilter, or queryContractId. The :: _ pattern silently picks the
/// first result from a non-deterministic query. The [(x)] pattern crashes on
/// 0 or 2+ results. Both are unsafe.
///
/// Catches: G5 (head-of-list on unordered query results)
pub struct HeadOfListQuery;

impl Detector for HeadOfListQuery {
    fn name(&self) -> &str {
        "head-of-list-query"
    }

    fn severity(&self) -> Severity {
        Severity::Medium
    }

    fn description(&self) -> &str {
        "Head-of-list pattern on query result (non-deterministic order)"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Scan all choice bodies and function bodies
        let all_bodies: Vec<(&str, &str, usize, &std::path::Path)> = {
            let mut bodies = Vec::new();
            for template in &module.templates {
                for choice in &template.choices {
                    bodies.push((
                        choice.body_raw.as_str(),
                        choice.name.as_str(),
                        choice.span.line,
                        module.file.as_path(),
                    ));
                }
            }
            for func in &module.functions {
                bodies.push((
                    func.body_raw.as_str(),
                    func.name.as_str(),
                    func.span.line,
                    module.file.as_path(),
                ));
            }
            bodies
        };

        for (body_raw, name, base_line, file) in all_bodies {
            let lines: Vec<&str> = body_raw.lines().collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Check for query/queryFilter/queryContractId usage
                let has_query = trimmed.contains("query ")
                    || trimmed.contains("queryFilter ")
                    || trimmed.contains("queryContractId ")
                    || trimmed.contains("queryInterface ");

                if !has_query {
                    continue;
                }

                // Look ahead for :: _ pattern or [(...)] pattern in subsequent lines
                let search_range = (line_idx + 1)..std::cmp::min(line_idx + 10, lines.len());
                for check_idx in search_range {
                    let check_line = lines[check_idx].trim();

                    // Pattern: (x, y) :: _ or someVar :: _
                    if check_line.contains(":: _") || check_line.contains("::_") {
                        findings.push(Finding {
                            detector: self.name().to_string(),
                            severity: self.severity(),
                            file: file.to_path_buf(),
                            line: base_line + check_idx,
                            column: 1,
                            message: format!(
                                "Head-of-list pattern ':: _' on query result in '{}'. \
                                 queryFilter returns results in non-deterministic order.",
                                name
                            ),
                            evidence: check_line.to_string(),
                        });
                        break;
                    }

                    // Pattern: [(x)] or [(x, y)]
                    if (check_line.starts_with("[(") && check_line.contains(")]"))
                        || check_line.contains("<- [(")
                    {
                        findings.push(Finding {
                            detector: self.name().to_string(),
                            severity: self.severity(),
                            file: file.to_path_buf(),
                            line: base_line + check_idx,
                            column: 1,
                            message: format!(
                                "Single-element list pattern on query result in '{}'. \
                                 Crashes on 0 or 2+ results.",
                                name
                            ),
                            evidence: check_line.to_string(),
                        });
                        break;
                    }
                }
            }

            // Also check the raw body for :: _ patterns directly after query lines
            // (handles case where query and pattern are on the same line)
            for (line_idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                let has_query = trimmed.contains("query")
                    || trimmed.contains("queryFilter")
                    || trimmed.contains("queryContractId");

                if has_query && (trimmed.contains(":: _") || trimmed.contains("::_")) {
                    findings.push(Finding {
                        detector: self.name().to_string(),
                        severity: self.severity(),
                        file: file.to_path_buf(),
                        line: base_line + line_idx,
                        column: 1,
                        message: format!(
                            "Head-of-list pattern on query result in '{}'. \
                             Query returns results in non-deterministic order.",
                            name
                        ),
                        evidence: trimmed.to_string(),
                    });
                }
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
    fn test_head_of_list_cons_pattern() {
        let source = r#"module Test where

getFeaturedAppRight owner = do
  results <- queryFilter @FeaturedAppRight (\r -> r.provider == owner)
  case results of
    (rightCid, _) :: _ -> do
      pure (Some rightCid)
    [] -> pure None
"#;
        let module = parse_daml(source, Path::new("AmuletRegistry.daml"));
        let findings = HeadOfListQuery.detect(&module);
        assert!(!findings.is_empty());
        assert!(findings[0].message.contains(":: _"));
    }

    #[test]
    fn test_head_of_list_singleton_pattern() {
        let source = r#"module Test where

getTransferFactory owner = do
  results <- query @TransferFactory owner
  case results of
    [(rulesCid, _)] -> pure rulesCid
"#;
        let module = parse_daml(source, Path::new("SimpleRegistry.daml"));
        let findings = HeadOfListQuery.detect(&module);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_safe_query_usage_passes() {
        let source = r#"module Test where

getAllFactories owner = do
  results <- query @TransferFactory owner
  mapA (\(cid, _) -> fetch cid) results
"#;
        let module = parse_daml(source, Path::new("Safe.daml"));
        let findings = HeadOfListQuery.detect(&module);
        assert!(findings.is_empty());
    }
}
