use crate::detector::{Detector, Finding, Severity};
use crate::ir::DamlModule;

/// Detector #7: observer-only-app-party
///
/// Opt-in detector for CIP-0104 traffic-based app rewards. Under CIP-0104, an
/// application earns rewards only on transactions where its party is a
/// confirming party (a signatory or a choice controller). A refactor that
/// demotes the app party to a plain observer keeps the templates compiling and
/// settling, but reward attribution silently stops.
///
/// The detector activates only when the operator names the app-party fields
/// with `--app-party` (repeatable). The check is module-level: a party
/// confirms when it appears in a `signatory` clause or in a choice
/// `controller` clause of any template in the module (a settling choice makes
/// the party a confirming party of the transaction that also creates the view
/// templates). The detector flags each named party that appears in an
/// `observer` clause somewhere in the module but confirms nowhere in it.
///
/// Matching is lexical with word boundaries: `--app-party app` matches
/// `observer app` and `controller settlement.executors` matches
/// `--app-party executors`, but `observer approver` does not match
/// `--app-party app`.
///
/// Some templates observe the app party on purpose, for example a pure
/// read-model view where the party must not confirm. Mark such a template
/// with a `-- daml-lint: allow=observer-only-app-party` annotation (in the
/// template body or on the line directly above the template header) and the
/// detector skips it. The finding is LOW severity: an observer-only app
/// party loses reward attribution on its own traffic, but no funds are at
/// risk.
pub struct ObserverOnlyAppParty {
    pub parties: Vec<String>,
}

/// True when `text` contains `name` as a whole word (the neighbor characters
/// are not identifier characters).
fn references_party(text: &str, name: &str) -> bool {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';
    let mut start = 0;
    while let Some(pos) = text[start..].find(name) {
        let abs = start + pos;
        let before_ok = abs == 0 || !text[..abs].chars().next_back().is_some_and(is_ident);
        let after = abs + name.len();
        let after_ok = after >= text.len() || !text[after..].chars().next().is_some_and(is_ident);
        if before_ok && after_ok {
            return true;
        }
        start = abs + name.len();
    }
    false
}

impl Detector for ObserverOnlyAppParty {
    fn name(&self) -> &str {
        "observer-only-app-party"
    }

    fn severity(&self) -> Severity {
        Severity::Low
    }

    fn description(&self) -> &str {
        "Configured app party is an observer but never a signatory or a choice controller, so it earns no CIP-0104 traffic-based rewards"
    }

    fn detect(&self, module: &DamlModule) -> Vec<Finding> {
        let mut findings = Vec::new();

        for party in &self.parties {
            let observing: Vec<_> = module
                .templates
                .iter()
                .filter(|t| !t.allowed_lints.iter().any(|a| a == self.name()))
                .filter(|t| t.observers.iter().any(|o| references_party(o, party)))
                .collect();
            if observing.is_empty() {
                continue;
            }

            let confirms_somewhere = module.templates.iter().any(|t| {
                t.signatories.iter().any(|s| references_party(s, party))
                    || t.choices.iter().any(|c| {
                        c.controllers
                            .iter()
                            .any(|ctrl| references_party(ctrl, party))
                    })
            });
            if confirms_somewhere {
                continue;
            }

            let template_names: Vec<&str> = observing.iter().map(|t| t.name.as_str()).collect();
            let first = observing[0];
            findings.push(Finding {
                detector: self.name().to_string(),
                severity: self.severity(),
                file: first.span.file.clone(),
                line: first.span.line,
                column: first.span.column,
                message: format!(
                    "Party '{}' is an observer on template(s) {} but is never a signatory or a choice controller in module '{}'. Under CIP-0104, only a confirming party earns traffic-based rewards; an observer-only app party earns nothing on these transactions. If the template observes the party on purpose, mark it with '-- daml-lint: allow=observer-only-app-party'.",
                    party,
                    template_names.join(", "),
                    module.name
                ),
                evidence: format!(
                    "observer {party}  -- not in any signatory or controller clause in the module"
                ),
            });
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_daml;
    use std::path::Path;

    fn detector() -> ObserverOnlyAppParty {
        ObserverOnlyAppParty {
            parties: vec!["app".to_string()],
        }
    }

    #[test]
    fn test_observer_only_app_party_triggers() {
        let source = r#"module Test where

template SettlementReceipt
  with
    admin : Party
    app : Party
    amount : Decimal
  where
    signatory admin
    observer app
    ensure amount > 0.0

    choice Acknowledge : ()
      controller admin
      do return ()
"#;
        let module = parse_daml(source, Path::new("Receipt.daml"));
        let findings = detector().detect(&module);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("SettlementReceipt"));
        assert!(findings[0].message.contains("'app'"));
        assert_eq!(findings[0].severity, Severity::Low);
    }

    #[test]
    fn test_allow_annotation_in_body_suppresses_the_template() {
        let source = r#"module Test where

template AuditView
  with
    admin : Party
    app : Party
  where
    signatory admin
    -- daml-lint: allow=observer-only-app-party
    observer app
"#;
        let module = parse_daml(source, Path::new("Audit.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_allow_annotation_above_header_suppresses_the_template() {
        let source = r#"module Test where

-- daml-lint: allow=observer-only-app-party
template AuditView
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app
"#;
        let module = parse_daml(source, Path::new("Audit.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_allow_annotation_only_suppresses_the_annotated_template() {
        let source = r#"module Test where

template AuditView
  with
    admin : Party
    app : Party
  where
    signatory admin
    -- daml-lint: allow=observer-only-app-party
    observer app

template SettlementReceipt
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app
"#;
        let module = parse_daml(source, Path::new("Mixed.daml"));
        let findings = detector().detect(&module);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("SettlementReceipt"));
        assert!(!findings[0].message.contains("AuditView"));
    }

    #[test]
    fn test_allow_annotation_for_another_detector_does_not_suppress() {
        let source = r#"module Test where

template SettlementReceipt
  with
    admin : Party
    app : Party
  where
    signatory admin
    -- daml-lint: allow=unbounded-fields
    observer app
"#;
        let module = parse_daml(source, Path::new("Receipt.daml"));
        let findings = detector().detect(&module);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_passes_when_party_controls_a_choice() {
        let source = r#"module Test where

template SettlementBatch
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app

    choice SettleBatch : ()
      controller app
      do return ()
"#;
        let module = parse_daml(source, Path::new("Batch.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_passes_when_party_is_signatory() {
        let source = r#"module Test where

template JointReceipt
  with
    admin : Party
    app : Party
  where
    signatory admin, app
    observer app
"#;
        let module = parse_daml(source, Path::new("Joint.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_findings_without_configuration() {
        let source = r#"module Test where

template SettlementReceipt
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app
"#;
        let module = parse_daml(source, Path::new("Receipt.daml"));
        let empty = ObserverOnlyAppParty { parties: vec![] };
        let findings = empty.detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_word_boundary_does_not_match_similar_names() {
        let source = r#"module Test where

template Review
  with
    admin : Party
    approver : Party
  where
    signatory admin
    observer approver
"#;
        let module = parse_daml(source, Path::new("Review.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_matches_party_inside_projection_expression() {
        let source = r#"module Test where

template Receipt
  with
    admin : Party
    executors : [Party]
  where
    signatory admin
    observer settlement.executors
"#;
        let module = parse_daml(source, Path::new("Receipt.daml"));
        let det = ObserverOnlyAppParty {
            parties: vec!["executors".to_string()],
        };
        let findings = det.detect(&module);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_passes_when_party_confirms_elsewhere_in_the_module() {
        // The receipt observes the app party. The factory choice makes the
        // app party a confirming party of the settling transaction. The
        // module-level check must not flag the receipt.
        let source = r#"module Test where

template SettlementReceipt
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app

template SettlementFactory
  with
    admin : Party
  where
    signatory admin

    choice SettleBatch : ()
      with
        app : Party
      controller app
      do return ()
"#;
        let module = parse_daml(source, Path::new("Settlement.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_one_finding_per_party_across_templates() {
        let source = r#"module Test where

template ReceiptA
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app

template ReceiptB
  with
    admin : Party
    app : Party
  where
    signatory admin
    observer app
"#;
        let module = parse_daml(source, Path::new("Receipts.daml"));
        let findings = detector().detect(&module);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("ReceiptA, ReceiptB"));
    }

    #[test]
    fn test_skips_templates_that_do_not_observe_the_party() {
        let source = r#"module Test where

template Unrelated
  with
    admin : Party
    owner : Party
  where
    signatory admin
    observer owner
"#;
        let module = parse_daml(source, Path::new("Unrelated.daml"));
        let findings = detector().detect(&module);
        assert!(findings.is_empty());
    }
}
