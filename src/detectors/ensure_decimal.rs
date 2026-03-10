use crate::detector::{Detector, Finding, Severity};
use crate::ir::DamlModule;

/// Detector #1: missing-ensure-decimal
///
/// For each template with a Decimal field, check that the template has an ensure
/// clause referencing that field with a > 0 or >= 0 comparison. Flags templates
/// where Decimal fields have no corresponding ensure bound.
///
/// Catches: G1 (missing ensure on monetary templates), M11 (round templates missing ensure)
pub struct MissingEnsureDecimal;

impl Detector for MissingEnsureDecimal {
    fn name(&self) -> &str {
        "missing-ensure-decimal"
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn description(&self) -> &str {
        "Template has Decimal field with no ensure clause bounding it to > 0"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        for template in &module.templates {
            let decimal_fields: Vec<_> = template
                .fields
                .iter()
                .filter(|f| f.type_.is_decimal())
                .collect();

            if decimal_fields.is_empty() {
                continue;
            }

            for field in &decimal_fields {
                let has_bound = template
                    .ensure_clause
                    .as_ref()
                    .is_some_and(|ec| ec.has_positive_bound(&field.name));

                if !has_bound {
                    let evidence = if template.ensure_clause.is_none() {
                        format!("{} : Decimal  -- no ensure clause found", field.name)
                    } else {
                        format!(
                            "{} : Decimal  -- ensure clause does not bound this field",
                            field.name
                        )
                    };

                    findings.push(Finding {
                        detector: self.name().to_string(),
                        severity: self.severity(),
                        file: template.span.file.clone(),
                        line: template.span.line,
                        column: template.span.column,
                        message: format!(
                            "Template '{}' has Decimal field '{}' with no ensure clause bounding it to > 0.",
                            template.name, field.name
                        ),
                        evidence,
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
    fn test_missing_ensure_decimal_triggers() {
        let source = r#"module Test where

template OpenMiningRound
  with
    admin : Party
    amuletPrice : Decimal
    tickDuration : Decimal
  where
    signatory admin
"#;
        let module = parse_daml(source, Path::new("Round.daml"));
        let findings = MissingEnsureDecimal.detect(&module);
        assert_eq!(findings.len(), 2);
        assert!(findings[0].message.contains("amuletPrice"));
        assert!(findings[1].message.contains("tickDuration"));
    }

    #[test]
    fn test_ensure_decimal_passes_with_bound() {
        let source = r#"module Test where

template SimpleHolding
  with
    admin : Party
    amount : Decimal
  where
    signatory admin
    ensure amount > 0.0
"#;
        let module = parse_daml(source, Path::new("Holding.daml"));
        let findings = MissingEnsureDecimal.detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_ensure_exists_but_doesnt_bound_field() {
        let source = r#"module Test where

template RoundWithPartialEnsure
  with
    admin : Party
    amuletPrice : Decimal
    tickDuration : Decimal
  where
    signatory admin
    ensure tickDuration > 0.0
"#;
        let module = parse_daml(source, Path::new("Round.daml"));
        let findings = MissingEnsureDecimal.detect(&module);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("amuletPrice"));
    }
}
