use crate::detector::{Detector, Finding, Severity};
use crate::ir::{DamlModule, Statement};

/// Detector #2: unguarded-division
///
/// Find all division expressions (/ operator or div function) in choice bodies
/// and functions. Walk backward through the statement list to find a prior
/// assertMsg or ensure check that bounds the denominator to > 0. Flag divisions
/// where no such guard exists.
///
/// Catches: M11 (amuletPrice division), M12 (capPerCoupon division)
pub struct UnguardedDivision;

impl UnguardedDivision {
    fn check_body_raw(
        &self,
        body_raw: &str,
        statements: &[Statement],
        file: &std::path::Path,
        base_line: usize,
        context_name: &str,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();
        let lines: Vec<&str> = body_raw.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Find division: look for / operator or `div` function
            let has_division = trimmed.contains(" / ")
                || trimmed.contains("(1.0 /")
                || trimmed.contains("(1 /")
                || trimmed.contains(" `div` ");

            if !has_division {
                continue;
            }

            // Extract the likely denominator
            let denominator = extract_denominator(trimmed);

            // Check if there's a prior guard in the statements
            let has_guard = self.has_prior_guard(&denominator, statements, body_raw, line_idx);

            if !has_guard {
                findings.push(Finding {
                    detector: self.name().to_string(),
                    severity: self.severity(),
                    file: file.to_path_buf(),
                    line: base_line + line_idx,
                    column: 1,
                    message: format!(
                        "Unguarded division by '{}' — no prior > 0 check found in {}.",
                        denominator, context_name
                    ),
                    evidence: trimmed.to_string(),
                });
            }
        }

        findings
    }

    fn has_prior_guard(
        &self,
        denominator: &str,
        statements: &[Statement],
        body_raw: &str,
        _division_line: usize,
    ) -> bool {
        if denominator.is_empty() {
            return false;
        }

        // Check if any assert or ensure references the denominator with a > 0 check
        for stmt in statements {
            match stmt {
                Statement::Assert { condition } => {
                    if condition.contains(denominator)
                        && (condition.contains("> 0")
                            || condition.contains(">= 0")
                            || condition.contains("/= 0")
                            || condition.contains("!= 0"))
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }

        // Also check the raw body for guard patterns
        let lines: Vec<&str> = body_raw.lines().collect();
        for line in &lines {
            let trimmed = line.trim();
            if (trimmed.contains("assertMsg") || trimmed.contains("assert "))
                && trimmed.contains(denominator)
                && (trimmed.contains("> 0")
                    || trimmed.contains(">= 0")
                    || trimmed.contains("/= 0")
                    || trimmed.contains("!= 0"))
            {
                return true;
            }
        }

        false
    }
}

fn extract_denominator(line: &str) -> String {
    // Try to extract the expression after / operator
    if let Some(idx) = line.find(" / ") {
        let after = &line[idx + 3..];
        let denom: String = after
            .trim()
            .trim_start_matches('(')
            .split(|c: char| c == ')' || c == ' ' || c == '\n')
            .next()
            .unwrap_or("")
            .to_string();
        return denom;
    }
    if let Some(idx) = line.find("(1.0 /") {
        let after = &line[idx + 6..];
        let denom: String = after
            .trim()
            .split(|c: char| c == ')' || c == '\n')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        return denom;
    }
    if let Some(idx) = line.find("(1 /") {
        let after = &line[idx + 4..];
        let denom: String = after
            .trim()
            .split(|c: char| c == ')' || c == '\n')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        return denom;
    }
    if let Some(idx) = line.find(" `div` ") {
        let after = &line[idx + 7..];
        let denom: String = after
            .trim()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        return denom;
    }
    String::new()
}

impl Detector for UnguardedDivision {
    fn name(&self) -> &str {
        "unguarded-division"
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn description(&self) -> &str {
        "Division without prior > 0 check on denominator"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        for template in &module.templates {
            for choice in &template.choices {
                findings.extend(self.check_body_raw(
                    &choice.body_raw,
                    &choice.body,
                    &module.file,
                    choice.span.line,
                    &format!("choice '{}'", choice.name),
                ));
            }
        }

        for func in &module.functions {
            findings.extend(self.check_body_raw(
                &func.body_raw,
                &func.body,
                &module.file,
                func.span.line,
                &format!("function '{}'", func.name),
            ));
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
    fn test_unguarded_division_triggers() {
        let source = r#"module Test where

scaleFees fees rate =
  map (\f -> f { amount = f.amount * (1.0 / rate)) fees
"#;
        let module = parse_daml(source, Path::new("AmuletRules.daml"));
        let findings = UnguardedDivision.detect(&module);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_guarded_division_passes() {
        let source = r#"module Test where

safeDivide x y = do
  assertMsg "denominator must be positive" (y > 0)
  pure (x / y)
"#;
        let module = parse_daml(source, Path::new("Safe.daml"));
        let findings = UnguardedDivision.detect(&module);
        assert!(findings.is_empty());
    }
}
