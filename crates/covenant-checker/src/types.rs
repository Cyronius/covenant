//! Resolved type representations

use std::collections::HashMap;
use covenant_ast::SymbolId;

/// A resolved type (after type checking)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedType {
    /// Primitive types
    Int,
    Float,
    Bool,
    String,
    Char,
    Bytes,
    DateTime,
    None,

    /// Named type with resolved ID
    Named {
        name: String,
        id: SymbolId,
        args: Vec<ResolvedType>,
    },

    /// Optional type
    Optional(Box<ResolvedType>),

    /// List type
    List(Box<ResolvedType>),

    /// Set type
    Set(Box<ResolvedType>),

    /// Union type
    Union(Vec<ResolvedType>),

    /// Tuple type
    Tuple(Vec<ResolvedType>),

    /// Function type
    Function {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
    },

    /// Struct type
    Struct(Vec<(String, ResolvedType)>),

    /// Unknown (for inference)
    Unknown,

    /// Error type (for error recovery)
    Error,
}

impl ResolvedType {
    pub fn is_error(&self) -> bool {
        matches!(self, ResolvedType::Error)
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, ResolvedType::Optional(_))
    }

    pub fn display(&self) -> String {
        match self {
            ResolvedType::Int => "Int".to_string(),
            ResolvedType::Float => "Float".to_string(),
            ResolvedType::Bool => "Bool".to_string(),
            ResolvedType::String => "String".to_string(),
            ResolvedType::Char => "Char".to_string(),
            ResolvedType::Bytes => "Bytes".to_string(),
            ResolvedType::DateTime => "DateTime".to_string(),
            ResolvedType::None => "none".to_string(),
            ResolvedType::Named { name, args, .. } => {
                if args.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        args.iter().map(|t| t.display()).collect::<Vec<_>>().join(", ")
                    )
                }
            }
            ResolvedType::Optional(inner) => format!("{}?", inner.display()),
            ResolvedType::List(inner) => format!("{}[]", inner.display()),
            ResolvedType::Set(inner) => format!("Set<{}>", inner.display()),
            ResolvedType::Union(types) => {
                types.iter().map(|t| t.display()).collect::<Vec<_>>().join(" | ")
            }
            ResolvedType::Tuple(types) => {
                format!(
                    "({})",
                    types.iter().map(|t| t.display()).collect::<Vec<_>>().join(", ")
                )
            }
            ResolvedType::Function { params, ret } => {
                format!(
                    "({}) -> {}",
                    params.iter().map(|t| t.display()).collect::<Vec<_>>().join(", "),
                    ret.display()
                )
            }
            ResolvedType::Struct(fields) => {
                format!(
                    "{{ {} }}",
                    fields
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ResolvedType::Unknown => "?".to_string(),
            ResolvedType::Error => "<error>".to_string(),
        }
    }
}

// ============================================================================
// Type Registry
// ============================================================================

/// Registry of struct and enum type definitions
#[derive(Debug, Default)]
pub struct TypeRegistry {
    /// Struct definitions: name -> fields
    structs: HashMap<String, StructDef>,
    /// Enum definitions: name -> variants
    enums: HashMap<String, EnumDef>,
}

/// Definition of a struct type
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, ResolvedType)>,
}

/// Definition of an enum type
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

/// Definition of an enum variant
#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    /// None for unit variants, Some for variants with fields
    pub fields: Option<Vec<(String, ResolvedType)>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a struct type definition
    pub fn register_struct(&mut self, name: String, fields: Vec<(String, ResolvedType)>) {
        self.structs.insert(name.clone(), StructDef { name, fields });
    }

    /// Register an enum type definition
    pub fn register_enum(&mut self, name: String, variants: Vec<VariantDef>) {
        self.enums.insert(name.clone(), EnumDef { name, variants });
    }

    /// Get a struct definition by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Get an enum definition by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums.get(name)
    }

    /// Get all variant names for an enum (for exhaustiveness checking)
    pub fn get_enum_variants(&self, name: &str) -> Option<Vec<String>> {
        self.enums
            .get(name)
            .map(|e| e.variants.iter().map(|v| v.name.clone()).collect())
    }

    /// Get field type from a struct
    pub fn get_struct_field(&self, struct_name: &str, field_name: &str) -> Option<&ResolvedType> {
        self.structs.get(struct_name).and_then(|s| {
            s.fields
                .iter()
                .find(|(name, _)| name == field_name)
                .map(|(_, ty)| ty)
        })
    }
}
