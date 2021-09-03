//!
//! The semantic analyzer scope error.
//!

use crate::lexical::token::location::Location;

#[derive(Debug, PartialEq)]
pub enum Error {
    ItemUndeclared {
        location: Location,
        name: String,
    },
    ItemRedeclared {
        location: Location,
        name: String,
        reference: Option<Location>,
    },
    ItemIsNotNamespace {
        location: Location,
        name: String,
    },
}
