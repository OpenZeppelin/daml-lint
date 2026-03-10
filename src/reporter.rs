use crate::detector::{Finding, Severity};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Sarif,
    Markdown,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sarif" => Some(OutputFormat::Sarif),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            "json" => Some(OutputFormat::Json),
            _ => None,
        }
    }
}

pub fn format_findings(findings: &[Finding], format: OutputFormat) -> String {
    match format {
        OutputFormat::Sarif => format_sarif(findings),
        OutputFormat::Markdown => format_markdown(findings),
        OutputFormat::Json => format_json(findings),
    }
}

fn format_sarif(findings: &[Finding]) -> String {
    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            json!({
                "ruleId": f.detector,
                "level": sarif_level(&f.severity),
                "message": {
                    "text": f.message,
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": f.file.display().to_string(),
                        },
                        "region": {
                            "startLine": f.line,
                            "startColumn": f.column,
                        }
                    }
                }],
                "properties": {
                    "evidence": f.evidence,
                }
            })
        })
        .collect();

    let rules: Vec<serde_json::Value> = {
        let mut seen = std::collections::HashSet::new();
        findings
            .iter()
            .filter(|f| seen.insert(f.detector.clone()))
            .map(|f| {
                json!({
                    "id": f.detector,
                    "shortDescription": {
                        "text": f.detector.replace('-', " "),
                    },
                    "defaultConfiguration": {
                        "level": sarif_level(&f.severity),
                    }
                })
            })
            .collect()
    };

    let sarif = json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "daml-lint",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/example/daml-lint",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap()
}

fn sarif_level(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Info => "note",
    }
}

fn format_markdown(findings: &[Finding]) -> String {
    let mut out = String::new();
    out.push_str("# daml-lint Report\n\n");

    if findings.is_empty() {
        out.push_str("No findings.\n");
        return out;
    }

    let (critical, high, medium, low, info) = count_by_severity(findings);
    out.push_str(&format!(
        "**Summary:** {} finding(s) — {} Critical, {} High, {} Medium, {} Low, {} Info\n\n",
        findings.len(),
        critical,
        high,
        medium,
        low,
        info
    ));

    // Group by severity
    for severity in &[
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ] {
        let group: Vec<_> = findings.iter().filter(|f| f.severity == *severity).collect();
        if group.is_empty() {
            continue;
        }

        out.push_str(&format!("## {} ({})\n\n", severity, group.len()));

        for f in &group {
            out.push_str(&format!(
                "### {} `{}`\n\n",
                f.severity, f.detector
            ));
            out.push_str(&format!("**{}**\n\n", f.message));
            out.push_str(&format!(
                "- **File:** `{}:{}`\n",
                f.file.display(),
                f.line
            ));
            out.push_str(&format!("- **Evidence:**\n  ```\n  {}\n  ```\n\n", f.evidence));
        }
    }

    out
}

fn format_json(findings: &[Finding]) -> String {
    #[derive(Serialize)]
    struct Report {
        tool: &'static str,
        version: &'static str,
        findings: Vec<FindingJson>,
        summary: Summary,
    }

    #[derive(Serialize)]
    struct FindingJson {
        detector: String,
        severity: String,
        file: String,
        line: usize,
        column: usize,
        message: String,
        evidence: String,
    }

    #[derive(Serialize)]
    struct Summary {
        total: usize,
        critical: usize,
        high: usize,
        medium: usize,
        low: usize,
        info: usize,
    }

    let (critical, high, medium, low, info) = count_by_severity(findings);

    let report = Report {
        tool: "daml-lint",
        version: env!("CARGO_PKG_VERSION"),
        findings: findings
            .iter()
            .map(|f| FindingJson {
                detector: f.detector.clone(),
                severity: f.severity.to_string(),
                file: f.file.display().to_string(),
                line: f.line,
                column: f.column,
                message: f.message.clone(),
                evidence: f.evidence.clone(),
            })
            .collect(),
        summary: Summary {
            total: findings.len(),
            critical,
            high,
            medium,
            low,
            info,
        },
    };

    serde_json::to_string_pretty(&report).unwrap()
}

fn count_by_severity(findings: &[Finding]) -> (usize, usize, usize, usize, usize) {
    let critical = findings.iter().filter(|f| f.severity == Severity::Critical).count();
    let high = findings.iter().filter(|f| f.severity == Severity::High).count();
    let medium = findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = findings.iter().filter(|f| f.severity == Severity::Low).count();
    let info = findings.iter().filter(|f| f.severity == Severity::Info).count();
    (critical, high, medium, low, info)
}

/// Returns exit code: 0 if no findings at or above the threshold, 1 otherwise.
pub fn exit_code(findings: &[Finding], fail_on: Severity) -> i32 {
    if findings.iter().any(|f| f.severity <= fail_on) {
        1
    } else {
        0
    }
}
