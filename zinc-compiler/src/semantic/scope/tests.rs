//!
//! The scope tests.
//!

#![cfg(test)]

use crate::error::Error;
use crate::lexical::token::lexeme::keyword::Keyword;
use crate::lexical::token::location::Location;
use crate::semantic::error::Error as SemanticError;
use crate::semantic::scope::error::Error as ScopeError;

#[test]
fn ok_current_scope() {
    let input = r#"
fn main() {
    const VALUE: u8 = 42;

    let result = VALUE;
}
"#;

    assert!(crate::semantic::tests::compile_entry(input).is_ok());
}

#[test]
fn ok_upper_scope() {
    let input = r#"
const VALUE: u8 = 42;

fn main() {
    let result = VALUE;
}
"#;

    assert!(crate::semantic::tests::compile_entry(input).is_ok());
}

#[test]
fn ok_same_function_name_in_different_scopes() {
    let input = r#"
    struct Struct1 {
        value: u32,
    }
    
    impl Struct1 {
        fn new( value: u32) -> Struct1 {
            Struct1 {
                value: value,
            }
        }
    
        fn empty() -> Struct1 {
            Struct1 {
                value: 0 as u32,
            }
        }
    }
    
    struct Struct2 {
        value: u32,
    }
    
    impl Struct2 {
        fn new( value: u32) -> Struct2 {
            Struct2 {
                value: value,
            }
        }
    
        fn empty() -> Struct2 {
            Struct2 {
                value: 0 as u32,
            }
        }
    }
    
    fn main(witness: Struct1, a: Struct2) {
        witness;
    }
"#;

    assert!(crate::semantic::tests::compile_entry(input).is_ok());
}

#[test]
fn ok_far_scope() {
    let input = r#"
const VALUE: u8 = 42;

fn main() {
    {
        {
            {
                {
                    let result = VALUE;
                }
            }
        }
    }
}
"#;

    assert!(crate::semantic::tests::compile_entry(input).is_ok());
}

#[test]
fn error_item_is_not_namespace() {
    let input = r#"
const NOT_NAMESPACE: u8 = 42;

fn main() {
    let result = NOT_NAMESPACE::UNDEFINED;
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Scope(
        ScopeError::ItemIsNotNamespace {
            location: Location::new(5, 18),
            name: "NOT_NAMESPACE".to_owned(),
        },
    )));

    let result = crate::semantic::tests::compile_entry(input);

    assert_eq!(result, expected);
}

#[test]
fn error_item_redeclared() {
    let input = r#"
fn main() {
    let result = 42;
    {
        let result = 69;
    }
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Scope(
        ScopeError::ItemRedeclared {
            location: Location::new(5, 13),
            name: "result".to_owned(),
            reference: Some(Location::new(3, 9)),
        },
    )));

    let result = crate::semantic::tests::compile_entry(input);

    assert_eq!(result, expected);
}

#[test]
fn error_item_undeclared() {
    let input = r#"
fn main() {
    result = 69;
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Scope(
        ScopeError::ItemUndeclared {
            location: Location::new(3, 5),
            name: "result".to_owned(),
        },
    )));

    let result = crate::semantic::tests::compile_entry(input);

    assert_eq!(result, expected);
}

#[test]
fn error_item_undeclared_lower() {
    let input = r#"
fn main() {
    {
        let result = 42;
    };
    result = 69;
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Scope(
        ScopeError::ItemUndeclared {
            location: Location::new(6, 5),
            name: "result".to_owned(),
        },
    )));

    let result = crate::semantic::tests::compile_entry(input);

    assert_eq!(result, expected);
}

#[test]
fn error_item_undeclared_enum_variant() {
    let input = r#"
enum Jabberwocky {
    Gone = 42,
}

fn main() {
    let really = Jabberwocky::Exists;
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Scope(
        ScopeError::ItemUndeclared {
            location: Location::new(7, 31),
            name: "Exists".to_owned(),
        },
    )));

    let result = crate::semantic::tests::compile_entry(input);

    assert_eq!(result, expected);
}
