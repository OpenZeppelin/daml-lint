use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DamlType {
    Party,
    Text,
    Decimal,
    Int,
    Bool,
    Date,
    Time,
    ContractId(Box<DamlType>),
    List(Box<DamlType>),
    Optional(Box<DamlType>),
    TextMap(Box<DamlType>),
    Map(Box<DamlType>, Box<DamlType>),
    Named(String),
    Unit,
    Unknown,
}

impl DamlType {
    pub fn from_str(s: &str) -> DamlType {
        let s = s.trim();
        if s == "Party" {
            DamlType::Party
        } else if s == "Text" {
            DamlType::Text
        } else if s == "Decimal" {
            DamlType::Decimal
        } else if s == "Int" {
            DamlType::Int
        } else if s == "Bool" {
            DamlType::Bool
        } else if s == "Date" {
            DamlType::Date
        } else if s == "Time" {
            DamlType::Time
        } else if s == "()" {
            DamlType::Unit
        } else if let Some(inner) = s.strip_prefix("ContractId ") {
            DamlType::ContractId(Box::new(DamlType::from_str(inner)))
        } else if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            DamlType::List(Box::new(DamlType::from_str(inner)))
        } else if let Some(inner) = s.strip_prefix("Optional ") {
            DamlType::Optional(Box::new(DamlType::from_str(inner)))
        } else if let Some(inner) = s.strip_prefix("TextMap ") {
            DamlType::TextMap(Box::new(DamlType::from_str(inner)))
        } else if s.starts_with(char::is_uppercase) {
            DamlType::Named(s.to_string())
        } else {
            DamlType::Unknown
        }
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self, DamlType::Decimal)
    }

    pub fn is_text(&self) -> bool {
        matches!(self, DamlType::Text)
    }

    pub fn is_textmap(&self) -> bool {
        matches!(self, DamlType::TextMap(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, DamlType::List(_))
    }

    pub fn is_unbounded(&self) -> bool {
        self.is_text() || self.is_textmap() || self.is_list()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub type_: DamlType,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Template {
    pub name: String,
    pub fields: Vec<Field>,
    pub signatories: Vec<String>,
    pub observers: Vec<String>,
    pub ensure_clause: Option<EnsureClause>,
    pub choices: Vec<Choice>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnsureClause {
    pub raw_text: String,
    pub span: Span,
}

impl EnsureClause {
    pub fn references_field_with_bound(&self, field_name: &str, bound: &str) -> bool {
        let text = &self.raw_text;
        text.contains(field_name) && text.contains(bound)
    }

    pub fn references_field(&self, field_name: &str) -> bool {
        self.raw_text.contains(field_name)
    }

    pub fn has_positive_bound(&self, field_name: &str) -> bool {
        let text = &self.raw_text;
        if !text.contains(field_name) {
            return false;
        }
        // Check for common positive bound patterns
        text.contains(&format!("{} > 0", field_name))
            || text.contains(&format!("{} > 0.0", field_name))
            || text.contains(&format!("{} >= 0", field_name))
            || text.contains(&format!("{} >= 0.0", field_name))
            || text.contains(&format!("0 < {}", field_name))
            || text.contains(&format!("0.0 < {}", field_name))
            || text.contains(&format!("0 <= {}", field_name))
            || text.contains(&format!("0.0 <= {}", field_name))
    }

    pub fn has_size_bound(&self, field_name: &str) -> bool {
        let text = &self.raw_text;
        if !text.contains(field_name) {
            return false;
        }
        text.contains(&format!("T.length {}", field_name))
            || text.contains(&format!("Text.length {}", field_name))
            || text.contains(&format!("length {}", field_name))
            || text.contains(&format!("Map.size {}", field_name))
            || text.contains(&format!("size {}", field_name))
            || text.contains(&format!("DA.Text.length {}", field_name))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Choice {
    pub name: String,
    pub consuming: bool,
    pub controllers: Vec<String>,
    pub parameters: Vec<Field>,
    pub return_type: DamlType,
    pub body: Vec<Statement>,
    pub body_raw: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum Statement {
    Let { name: String, expr: String },
    Assert { condition: String },
    Fetch { cid_expr: String },
    Archive { cid_expr: String },
    Create { template_name: String, raw: String },
    Exercise { cid_expr: String, choice_name: String, raw: String },
    TryCatch { try_body: Vec<Statement>, catch_body: Vec<Statement> },
    Other { raw: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
    pub body_raw: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Import {
    pub module_name: String,
    pub qualified: bool,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DamlModule {
    pub name: String,
    pub file: PathBuf,
    pub imports: Vec<Import>,
    pub templates: Vec<Template>,
    pub functions: Vec<Function>,
}
