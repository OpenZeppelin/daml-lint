use crate::detector::{Detector, Finding, Severity};
use crate::ir::DamlModule;

/// Detector #5: missing-positive-amount
///
/// For each choice that accepts a Decimal parameter named amount (or a record
/// field containing amount : Decimal), check that the choice body contains an
/// assertMsg comparing amount > 0. Also check for list parameters named
/// inputHoldingCids or similar — flag if there is no `not $ null` check.
///
/// Catches: G2 (missing positive-amount check), H2 (zero-input transfer)
pub struct MissingPositiveAmount;

impl Detector for MissingPositiveAmount {
    fn name(&self) -> &str {
        "missing-positive-amount"
    }

    fn severity(&self) -> Severity {
        Severity::High
    }

    fn description(&self) -> &str {
        "Choice accepts amount/transfer parameter without positive-value or non-empty check"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        for template in &module.templates {
            for choice in &template.choices {
                // Check for Decimal parameters named "amount" or containing "amount"
                let amount_params: Vec<_> = choice
                    .parameters
                    .iter()
                    .filter(|p| {
                        p.type_.is_decimal()
                            && (p.name.to_lowercase().contains("amount")
                                || p.name.to_lowercase() == "quantity"
                                || p.name.to_lowercase() == "price")
                    })
                    .collect();

                for param in &amount_params {
                    let has_check = choice.body_raw.contains(&format!("{} > 0", param.name))
                        || choice.body_raw.contains(&format!("{} > 0.0", param.name))
                        || choice.body_raw.contains(&format!("{} >= 0", param.name))
                        || choice.body_raw.contains(&format!("{} >= 0.0", param.name));

                    if !has_check {
                        findings.push(Finding {
                            detector: self.name().to_string(),
                            severity: self.severity(),
                            file: choice.span.file.clone(),
                            line: choice.span.line,
                            column: choice.span.column,
                            message: format!(
                                "Choice '{}' accepts Decimal parameter '{}' without asserting > 0.",
                                choice.name, param.name
                            ),
                            evidence: format!("{} : Decimal  -- no positive-amount check", param.name),
                        });
                    }
                }

                // Check for list parameters that should be non-empty
                // Look for patterns like inputHoldingCids, inputs, etc.
                let list_params: Vec<_> = choice
                    .parameters
                    .iter()
                    .filter(|p| {
                        p.type_.is_list()
                            && (p.name.to_lowercase().contains("input")
                                || p.name.to_lowercase().contains("holding")
                                || p.name.to_lowercase().contains("cids"))
                    })
                    .collect();

                for param in &list_params {
                    let has_null_check = choice.body_raw.contains(&format!("null {}", param.name))
                        || choice.body_raw.contains(&format!("null transfer.{}", param.name))
                        || choice.body_raw.contains(&format!("length {}", param.name))
                        || choice.body_raw.contains(&format!("length transfer.{}", param.name))
                        || choice.body_raw.contains("not $ null");

                    if !has_null_check {
                        findings.push(Finding {
                            detector: self.name().to_string(),
                            severity: self.severity(),
                            file: choice.span.file.clone(),
                            line: choice.span.line,
                            column: choice.span.column,
                            message: format!(
                                "Choice '{}' accepts list parameter '{}' but has no minimum-length check.",
                                choice.name, param.name
                            ),
                            evidence: format!("No 'not $ null {}' or length check", param.name),
                        });
                    }
                }

                // Also scan body_raw for transfer-related patterns:
                // Look for ".inputs" or ".inputHoldingCids" usage without null checks
                if choice.body_raw.contains("inputHoldingCids")
                    || choice.body_raw.contains(".inputs")
                {
                    let has_min_check = choice.body_raw.contains("not $ null")
                        || choice.body_raw.contains("length transfer.inputs")
                        || choice.body_raw.contains("length transfer.inputHoldingCids")
                        || choice.body_raw.contains("null transfer.inputs")
                        || choice.body_raw.contains("null transfer.inputHoldingCids");

                    let has_max_only = choice.body_raw.contains("maxNumInputs")
                        || choice.body_raw.contains("< maxNum");

                    if !has_min_check && has_max_only {
                        findings.push(Finding {
                            detector: self.name().to_string(),
                            severity: self.severity(),
                            file: choice.span.file.clone(),
                            line: choice.span.line,
                            column: choice.span.column,
                            message: format!(
                                "Choice '{}' checks max input count but not min. Empty inputs allowed.",
                                choice.name,
                            ),
                            evidence: "Checks maxNumInputs but no 'not $ null' or min length guard".to_string(),
                        });
                    }
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
    fn test_missing_positive_amount_triggers() {
        let source = r#"module Test where

template Token
  with
    owner : Party
  where
    signatory owner

    choice Transfer : ContractId Token
      with
        amount : Decimal
        newOwner : Party
      controller owner
      do
        create this with owner = newOwner
"#;
        let module = parse_daml(source, Path::new("Token.daml"));
        let findings = MissingPositiveAmount.detect(&module);
        assert!(!findings.is_empty());
        assert!(findings[0].message.contains("amount"));
    }

    #[test]
    fn test_positive_amount_check_passes() {
        let source = r#"module Test where

template Token
  with
    owner : Party
  where
    signatory owner

    choice Transfer : ContractId Token
      with
        amount : Decimal
        newOwner : Party
      controller owner
      do
        assertMsg "amount must be positive" (amount > 0.0)
        create this with owner = newOwner
"#;
        let module = parse_daml(source, Path::new("Token.daml"));
        let findings = MissingPositiveAmount.detect(&module);
        assert!(findings.is_empty());
    }
}
