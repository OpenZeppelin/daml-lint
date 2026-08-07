use crate::ir::*;
use std::path::Path;

/// Parse a DAML source file into a DamlModule IR.
///
/// Uses tree-sitter-haskell for validation, then a line-based DAML keyword shim
/// to extract templates, choices, fields, and ensure clauses. tree-sitter-haskell
/// treats DAML keywords as identifiers, so structural extraction is done by
/// matching indentation-based patterns in the source text.
pub fn parse_daml(source: &str, file: &Path) -> DamlModule {
    // Validate with tree-sitter (best-effort — DAML may have parse errors)
    let mut ts_parser = tree_sitter::Parser::new();
    ts_parser
        .set_language(&tree_sitter_haskell::LANGUAGE.into())
        .expect("Error loading Haskell parser");
    let _tree = ts_parser.parse(source, None);

    let lines: Vec<&str> = source.lines().collect();
    let module_name = extract_module_name(&lines);
    let imports = extract_imports(&lines);
    let templates = extract_templates(&lines, file);
    let functions = extract_functions(&lines, file, &templates);

    DamlModule {
        name: module_name,
        file: file.to_path_buf(),
        imports,
        templates,
        functions,
    }
}

fn extract_module_name(lines: &[&str]) -> String {
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            if let Some(name) = trimmed
                .strip_prefix("module ")
                .and_then(|s| s.split_whitespace().next())
            {
                return name.to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn extract_imports(lines: &[&str]) -> Vec<Import> {
    let mut imports = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            let qualified = trimmed.contains("qualified");
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let module_name = parts
                .iter()
                .find(|p| {
                    p.starts_with(|c: char| c.is_uppercase()) && **p != "qualified"
                })
                .unwrap_or(&"Unknown")
                .to_string();
            let alias = parts
                .iter()
                .position(|p| *p == "as")
                .and_then(|i| parts.get(i + 1))
                .map(|s| s.to_string());
            imports.push(Import {
                module_name,
                qualified,
                alias,
            });
        }
    }
    imports
}

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn extract_templates(lines: &[&str], file: &Path) -> Vec<Template> {
    let mut templates = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("template ")
            && !trimmed.starts_with("template instance")
        {
            let template_indent = indent_level(lines[i]);
            let name = trimmed
                .strip_prefix("template ")
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap_or("Unknown")
                .to_string();
            let span = Span {
                file: file.to_path_buf(),
                line: i + 1,
                column: template_indent + 1,
            };

            // Find the template body (everything indented more than template line)
            let body_start = i + 1;
            let mut body_end = body_start;
            while body_end < lines.len() {
                if lines[body_end].trim().is_empty() {
                    body_end += 1;
                    continue;
                }
                if indent_level(lines[body_end]) <= template_indent
                    && !lines[body_end].trim().is_empty()
                {
                    break;
                }
                body_end += 1;
            }

            let template_body = &lines[body_start..body_end];
            let fields = extract_fields(template_body, body_start, file);
            let signatories = extract_clause(template_body, "signatory");
            let observers = extract_clause(template_body, "observer");
            let ensure_clause = extract_ensure(template_body, body_start, file);
            let choices = extract_choices(template_body, body_start, file);

            templates.push(Template {
                name,
                fields,
                signatories,
                observers,
                ensure_clause,
                choices,
                span,
            });

            i = body_end;
        } else {
            i += 1;
        }
    }

    templates
}

fn extract_fields(body: &[&str], body_offset: usize, file: &Path) -> Vec<Field> {
    let mut fields = Vec::new();
    let mut in_with_block = false;
    let mut found_first_with = false;

    for (idx, line) in body.iter().enumerate() {
        let trimmed = line.trim();

        // Only match the first `with` block (template fields), not choice `with` blocks
        if !found_first_with && (trimmed == "with" || trimmed.starts_with("with") && trimmed.len() == 4) {
            in_with_block = true;
            found_first_with = true;
            continue;
        }

        // End of with block when we hit where, signatory, ensure, choice, etc.
        if in_with_block
            && (trimmed.starts_with("where")
                || trimmed.starts_with("signatory")
                || trimmed.starts_with("observer")
                || trimmed.starts_with("ensure")
                || trimmed.starts_with("choice")
                || trimmed.starts_with("key")
                || trimmed.starts_with("maintainer"))
        {
            in_with_block = false;
        }

        if in_with_block && trimmed.contains(" : ") {
            let parts: Vec<&str> = trimmed.splitn(2, " : ").collect();
            if parts.len() == 2 {
                let field_name = parts[0].trim().to_string();
                let type_str = parts[1].trim();
                // Skip lines that look like type signatures for functions
                if !field_name.contains(' ') && !field_name.is_empty() {
                    fields.push(Field {
                        name: field_name,
                        type_: DamlType::from_str(type_str),
                        span: Span {
                            file: file.to_path_buf(),
                            line: body_offset + idx + 1,
                            column: indent_level(line) + 1,
                        },
                    });
                }
            }
        }
    }

    fields
}

fn extract_clause(body: &[&str], keyword: &str) -> Vec<String> {
    let mut results = Vec::new();
    for line in body {
        let trimmed = line.trim();
        if trimmed.starts_with(keyword) {
            let rest = trimmed[keyword.len()..].trim();
            // Parse party expressions: could be `admin`, `[admin, user]`, etc.
            let parties: Vec<String> = rest
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            results.extend(parties);
        }
    }
    results
}

fn extract_ensure(body: &[&str], body_offset: usize, file: &Path) -> Option<EnsureClause> {
    for (idx, line) in body.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("ensure ") || trimmed == "ensure" {
            // Collect the ensure clause which may span multiple lines
            let ensure_indent = indent_level(line);
            let mut raw_text = trimmed.to_string();
            let mut j = idx + 1;
            while j < body.len() {
                let next_trimmed = body[j].trim();
                if next_trimmed.is_empty() {
                    j += 1;
                    continue;
                }
                if indent_level(body[j]) > ensure_indent {
                    raw_text.push(' ');
                    raw_text.push_str(next_trimmed);
                    j += 1;
                } else {
                    break;
                }
            }
            return Some(EnsureClause {
                raw_text,
                span: Span {
                    file: file.to_path_buf(),
                    line: body_offset + idx + 1,
                    column: indent_level(line) + 1,
                },
            });
        }
    }
    None
}

fn extract_choices(body: &[&str], body_offset: usize, file: &Path) -> Vec<Choice> {
    let mut choices = Vec::new();
    let mut i = 0;

    while i < body.len() {
        let trimmed = body[i].trim();

        // Detect consuming modifiers
        let (is_consuming, choice_line) = if trimmed.starts_with("nonconsuming choice ")
            || trimmed.starts_with("preconsuming choice ")
            || trimmed.starts_with("postconsuming choice ")
        {
            (false, trimmed)
        } else if trimmed.starts_with("choice ") {
            (true, trimmed)
        } else {
            i += 1;
            continue;
        };

        // Parse "choice Name : ReturnType"
        let after_choice = if let Some(rest) = choice_line.strip_prefix("nonconsuming choice ") {
            rest
        } else if let Some(rest) = choice_line.strip_prefix("preconsuming choice ") {
            rest
        } else if let Some(rest) = choice_line.strip_prefix("postconsuming choice ") {
            rest
        } else {
            choice_line.strip_prefix("choice ").unwrap()
        };

        let (choice_name, return_type) = if after_choice.contains(" : ") {
            let parts: Vec<&str> = after_choice.splitn(2, " : ").collect();
            (
                parts[0].trim().to_string(),
                DamlType::from_str(parts[1].trim()),
            )
        } else {
            (after_choice.trim().to_string(), DamlType::Unknown)
        };

        let choice_indent = indent_level(body[i]);
        let span = Span {
            file: file.to_path_buf(),
            line: body_offset + i + 1,
            column: choice_indent + 1,
        };

        // Collect the choice body
        let choice_start = i + 1;
        let mut choice_end = choice_start;
        while choice_end < body.len() {
            if body[choice_end].trim().is_empty() {
                choice_end += 1;
                continue;
            }
            if indent_level(body[choice_end]) <= choice_indent
                && !body[choice_end].trim().is_empty()
            {
                break;
            }
            choice_end += 1;
        }

        let choice_body = &body[choice_start..choice_end];
        let parameters = extract_choice_params(choice_body, body_offset + choice_start, file);
        let controllers = extract_clause(choice_body, "controller");
        let body_raw = choice_body.iter().map(|l| *l).collect::<Vec<&str>>().join("\n");
        let statements = extract_statements(&body_raw);

        choices.push(Choice {
            name: choice_name,
            consuming: is_consuming,
            controllers,
            parameters,
            return_type,
            body: statements,
            body_raw,
            span,
        });

        i = choice_end;
    }

    choices
}

fn extract_choice_params(body: &[&str], body_offset: usize, file: &Path) -> Vec<Field> {
    let mut fields = Vec::new();
    let mut in_with = false;
    let mut with_indent = 0;

    for (idx, line) in body.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed == "with" {
            in_with = true;
            with_indent = indent_level(line);
            continue;
        }

        if in_with {
            if trimmed.starts_with("controller")
                || trimmed.starts_with("do")
                || (indent_level(line) <= with_indent && !trimmed.is_empty())
            {
                in_with = false;
                continue;
            }

            if trimmed.contains(" : ") {
                let parts: Vec<&str> = trimmed.splitn(2, " : ").collect();
                if parts.len() == 2 {
                    let field_name = parts[0].trim().to_string();
                    if !field_name.contains(' ') && !field_name.is_empty() {
                        fields.push(Field {
                            name: field_name,
                            type_: DamlType::from_str(parts[1].trim()),
                            span: Span {
                                file: file.to_path_buf(),
                                line: body_offset + idx + 1,
                                column: indent_level(line) + 1,
                            },
                        });
                    }
                }
            }
        }
    }

    fields
}

fn extract_statements(body_raw: &str) -> Vec<Statement> {
    let mut statements = Vec::new();
    let lines: Vec<&str> = body_raw.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.is_empty()
            || trimmed.starts_with("with")
            || trimmed.starts_with("controller")
            || trimmed.starts_with("do")
            || trimmed.contains(" : ") && !trimmed.contains("<-")
        {
            i += 1;
            continue;
        }

        if trimmed.starts_with("let ") {
            let rest = trimmed.strip_prefix("let ").unwrap();
            if let Some(eq_pos) = rest.find('=') {
                let name = rest[..eq_pos].trim().to_string();
                let expr = rest[eq_pos + 1..].trim().to_string();
                statements.push(Statement::Let { name, expr });
            }
        } else if trimmed.starts_with("assertMsg") || trimmed.starts_with("assert ") {
            statements.push(Statement::Assert {
                condition: trimmed.to_string(),
            });
        } else if trimmed.contains("fetchAndArchive") || (trimmed.contains("fetch") && !trimmed.contains("fetchByKey")) {
            if trimmed.contains("fetchAndArchive") {
                let cid = trimmed
                    .split("fetchAndArchive")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                statements.push(Statement::Archive { cid_expr: cid.clone() });
                statements.push(Statement::Fetch { cid_expr: cid });
            } else if trimmed.contains("<-") && trimmed.contains("fetch ") {
                let cid = trimmed
                    .split("fetch ")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                statements.push(Statement::Fetch { cid_expr: cid });
            }
        } else if trimmed.starts_with("archive ") || trimmed.contains("<- archive ") {
            let cid = trimmed
                .split("archive ")
                .nth(1)
                .unwrap_or("")
                .trim()
                .to_string();
            statements.push(Statement::Archive { cid_expr: cid });
        } else if trimmed.contains("create ") && !trimmed.contains("create this") || trimmed.contains("create this") {
            statements.push(Statement::Create {
                template_name: String::new(),
                raw: trimmed.to_string(),
            });
        } else if trimmed.contains("exercise ") {
            statements.push(Statement::Exercise {
                cid_expr: String::new(),
                choice_name: String::new(),
                raw: trimmed.to_string(),
            });
        } else if trimmed.starts_with("try") || trimmed == "try" {
            // Collect try body
            let try_indent = indent_level(lines[i]);
            let mut try_body_lines = Vec::new();
            let mut catch_body_lines = Vec::new();
            let mut in_catch = false;
            let mut j = i + 1;
            while j < lines.len() {
                let inner_trimmed = lines[j].trim();
                if inner_trimmed.starts_with("catch") {
                    in_catch = true;
                    j += 1;
                    continue;
                }
                if !inner_trimmed.is_empty()
                    && indent_level(lines[j]) <= try_indent
                    && !in_catch
                {
                    break;
                }
                if in_catch {
                    catch_body_lines.push(lines[j]);
                } else {
                    try_body_lines.push(lines[j]);
                }
                j += 1;
            }
            let try_raw = try_body_lines.join("\n");
            let catch_raw = catch_body_lines.join("\n");
            statements.push(Statement::TryCatch {
                try_body: extract_statements(&try_raw),
                catch_body: extract_statements(&catch_raw),
            });
            i = j;
            continue;
        } else {
            statements.push(Statement::Other {
                raw: trimmed.to_string(),
            });
        }

        i += 1;
    }

    statements
}

fn extract_functions(lines: &[&str], file: &Path, _templates: &[Template]) -> Vec<Function> {
    let mut functions = Vec::new();
    let mut i = 0;

    // Collect line ranges that are inside templates to skip them
    let mut template_ranges: Vec<(usize, usize)> = Vec::new();
    {
        let mut ti = 0;
        while ti < lines.len() {
            let trimmed = lines[ti].trim();
            if trimmed.starts_with("template ") && !trimmed.starts_with("template instance") {
                let template_indent = indent_level(lines[ti]);
                let start = ti;
                ti += 1;
                while ti < lines.len() {
                    if lines[ti].trim().is_empty() {
                        ti += 1;
                        continue;
                    }
                    if indent_level(lines[ti]) <= template_indent
                        && !lines[ti].trim().is_empty()
                    {
                        break;
                    }
                    ti += 1;
                }
                template_ranges.push((start, ti));
            } else {
                ti += 1;
            }
        }
    }

    let in_template = |line_idx: usize| -> bool {
        template_ranges.iter().any(|(s, e)| line_idx >= *s && line_idx < *e)
    };

    while i < lines.len() {
        if in_template(i) {
            i += 1;
            continue;
        }

        let trimmed = lines[i].trim();

        // Look for top-level function definitions: name ... = ...
        // or name arg1 arg2 = ...
        if !trimmed.is_empty()
            && !trimmed.starts_with("module ")
            && !trimmed.starts_with("import ")
            && !trimmed.starts_with("--")
            && !trimmed.starts_with("{-")
            && !trimmed.starts_with("template ")
            && indent_level(lines[i]) == 0
            && trimmed.contains(" = ")
            || (indent_level(lines[i]) == 0
                && trimmed.contains('=')
                && !trimmed.starts_with("module")
                && !trimmed.starts_with("import")
                && !trimmed.starts_with("--")
                && !trimmed.starts_with("template")
                && !in_template(i))
        {
            let name = trimmed.split_whitespace().next().unwrap_or("").to_string();
            if name.is_empty()
                || name.starts_with(|c: char| c.is_uppercase())
                || name == "type"
                || name == "data"
                || name == "class"
                || name == "instance"
                || name == "deriving"
            {
                i += 1;
                continue;
            }

            let func_start = i;
            let mut func_end = i + 1;
            while func_end < lines.len() {
                if lines[func_end].trim().is_empty() {
                    func_end += 1;
                    continue;
                }
                if indent_level(lines[func_end]) == 0 && !lines[func_end].trim().is_empty() {
                    break;
                }
                func_end += 1;
            }

            let body_raw = lines[func_start..func_end].join("\n");
            let statements = extract_statements(&body_raw);

            functions.push(Function {
                name,
                body: statements,
                body_raw,
                span: Span {
                    file: file.to_path_buf(),
                    line: func_start + 1,
                    column: 1,
                },
            });

            i = func_end;
        } else {
            i += 1;
        }
    }

    functions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_simple_template() {
        let source = r#"module Test where

template SimpleHolding
  with
    admin : Party
    amount : Decimal
  where
    signatory admin
    ensure amount > 0.0

    choice Transfer : ContractId SimpleHolding
      with
        newOwner : Party
      controller admin
      do
        create this with admin = newOwner
"#;
        let module = parse_daml(source, Path::new("Test.daml"));
        assert_eq!(module.name, "Test");
        assert_eq!(module.templates.len(), 1);

        let t = &module.templates[0];
        assert_eq!(t.name, "SimpleHolding");
        assert_eq!(t.fields.len(), 2);
        assert_eq!(t.fields[0].name, "admin");
        assert!(matches!(t.fields[0].type_, DamlType::Party));
        assert_eq!(t.fields[1].name, "amount");
        assert!(t.fields[1].type_.is_decimal());
        assert!(t.ensure_clause.is_some());
        assert!(t.ensure_clause.as_ref().unwrap().raw_text.contains("amount > 0.0"));
        assert_eq!(t.choices.len(), 1);
        assert_eq!(t.choices[0].name, "Transfer");
        assert_eq!(t.choices[0].parameters.len(), 1);
    }

    #[test]
    fn test_parse_template_without_ensure() {
        let source = r#"module Test where

template OpenMiningRound
  with
    admin : Party
    amuletPrice : Decimal
    tickDuration : RelTime
  where
    signatory admin
"#;
        let module = parse_daml(source, Path::new("Round.daml"));
        assert_eq!(module.templates.len(), 1);
        let t = &module.templates[0];
        assert_eq!(t.name, "OpenMiningRound");
        assert!(t.ensure_clause.is_none());
        assert_eq!(t.fields.len(), 3);
        assert!(t.fields[1].type_.is_decimal());
    }

    #[test]
    fn test_parse_nonconsuming_choice() {
        let source = r#"module Test where

template Foo
  with
    owner : Party
  where
    signatory owner

    nonconsuming choice GetInfo : Text
      controller owner
      do
        pure "info"
"#;
        let module = parse_daml(source, Path::new("Foo.daml"));
        assert_eq!(module.templates[0].choices.len(), 1);
        assert!(!module.templates[0].choices[0].consuming);
    }
}
