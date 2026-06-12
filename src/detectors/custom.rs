use crate::detector::{parse_severity, Detector, Finding, Severity};
use crate::ir::DamlModule;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

/// Custom detector: user-defined regex rule loaded from a JSON file via --rules.
///
/// Each rule scans every raw source line (including comments and string
/// literals) and reports one finding per match. Rule file format (a JSON
/// array; description is optional):
///
/// [
///   {
///     "name": "no-trace",
///     "severity": "low",
///     "description": "Debug trace left in code",
///     "pattern": "\\btrace\\b",
///     "message": "Remove debug trace calls before deploying"
///   }
/// ]
#[derive(Deserialize)]
struct RawRule {
    name: String,
    severity: String,
    #[serde(default)]
    description: String,
    pattern: String,
    message: String,
}

pub struct CustomDetector {
    name: String,
    severity: Severity,
    description: String,
    pattern: Regex,
    message: String,
}

pub fn load_rules(path: &Path) -> Result<Vec<Box<dyn Detector>>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read rules file {}: {}", path.display(), e))?;
    let raw: Vec<RawRule> = serde_json::from_str(&text)
        .map_err(|e| format!("invalid rules file {}: {}", path.display(), e))?;

    let mut seen: HashSet<String> = crate::detector::all_detectors()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    raw.into_iter()
        .map(|r| {
            if !seen.insert(r.name.clone()) {
                return Err(format!(
                    "rule '{}': name collides with a built-in detector or another rule",
                    r.name
                ));
            }
            let severity = parse_severity(&r.severity).ok_or_else(|| {
                format!(
                    "rule '{}': unknown severity '{}'. Use critical, high, medium, low, or info.",
                    r.name, r.severity
                )
            })?;
            let pattern = Regex::new(&r.pattern)
                .map_err(|e| format!("rule '{}': invalid pattern: {}", r.name, e))?;
            Ok(Box::new(CustomDetector {
                name: r.name,
                severity,
                description: r.description,
                pattern,
                message: r.message,
            }) as Box<dyn Detector>)
        })
        .collect()
}

impl Detector for CustomDetector {
    fn name(&self) -> &str {
        &self.name
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (idx, line) in module.source.lines().enumerate() {
            for m in self.pattern.find_iter(line) {
                findings.push(Finding {
                    detector: self.name.clone(),
                    severity: self.severity,
                    file: module.file.clone(),
                    line: idx + 1,
                    column: m.start() + 1,
                    message: self.message.clone(),
                    evidence: line.trim().to_string(),
                });
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

    fn demo_detector() -> CustomDetector {
        CustomDetector {
            name: "no-trace".to_string(),
            severity: Severity::Low,
            description: "Debug trace left in code".to_string(),
            pattern: Regex::new(r"\btrace\b").unwrap(),
            message: "Remove debug trace calls before deploying".to_string(),
        }
    }

    #[test]
    fn test_custom_rule_triggers() {
        let source = r#"module Test where

logPrice price = trace ("price: " <> show price) price
"#;
        let module = parse_daml(source, Path::new("Test.daml"));
        let findings = demo_detector().detect(&module);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 3);
        assert_eq!(findings[0].detector, "no-trace");
    }

    #[test]
    fn test_custom_rule_passes_clean_code() {
        let source = r#"module Test where

logPrice price = price
"#;
        let module = parse_daml(source, Path::new("Test.daml"));
        let findings = demo_detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_multiple_matches_on_one_line_all_reported() {
        let source = r#"module Test where

logBoth x = trace "a" (trace "b" x)
"#;
        let module = parse_daml(source, Path::new("Test.daml"));
        let findings = demo_detector().detect(&module);
        assert_eq!(findings.len(), 2);
    }

    fn load_rules_from_str(label: &str, json: &str) -> Result<Vec<Box<dyn Detector>>, String> {
        let path = std::env::temp_dir().join(format!(
            "daml-lint-test-{}-{}.json",
            label,
            std::process::id()
        ));
        std::fs::write(&path, json).unwrap();
        let result = load_rules(&path);
        std::fs::remove_file(&path).ok();
        result
    }

    #[test]
    fn test_load_rules_rejects_bad_severity() {
        let result = load_rules_from_str(
            "bad-severity",
            r#"[{"name": "x", "severity": "huge", "pattern": "x", "message": "m"}]"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_load_rules_rejects_builtin_name_collision() {
        let result = load_rules_from_str(
            "builtin-collision",
            r#"[{"name": "unguarded-division", "severity": "low", "pattern": "x", "message": "m"}]"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_load_rules_rejects_duplicate_rule_names() {
        let result = load_rules_from_str(
            "dup-names",
            r#"[{"name": "a", "severity": "low", "pattern": "x", "message": "m"},
                {"name": "a", "severity": "low", "pattern": "y", "message": "m"}]"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_load_rules_demo_file() {
        let detectors = load_rules(Path::new("examples/custom-rules.json")).unwrap();
        assert!(!detectors.is_empty());
    }
}
