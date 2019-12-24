//!
//! The match expression.
//!

mod builder;

pub use self::builder::Builder;

use crate::lexical::Location;
use crate::syntax;
use crate::syntax::MatchPattern;

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub location: Location,
    pub scrutinee: syntax::Expression,
    pub branches: Vec<(MatchPattern, syntax::Expression)>,
}

impl Expression {
    pub fn new(
        location: Location,
        scrutinee: syntax::Expression,
        branches: Vec<(MatchPattern, syntax::Expression)>,
    ) -> Self {
        Self {
            location,
            scrutinee,
            branches,
        }
    }
}
