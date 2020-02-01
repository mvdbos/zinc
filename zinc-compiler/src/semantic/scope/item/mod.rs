//!
//! The semantic analyzer scope item.
//!

mod r#static;
mod variable;

pub use self::r#static::Static;
pub use self::variable::Variable;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::semantic::Constant;
use crate::semantic::Scope;
use crate::semantic::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Variable(Variable),
    Constant(Constant),
    Static(Static),
    Type(Type),
    Module(Rc<RefCell<Scope>>),
}

impl Item {
    pub fn is_namespace(&self) -> bool {
        match self {
            Self::Variable(_) => false,
            Self::Constant(_) => false,
            Self::Static(_) => false,
            Self::Type(Type::Enumeration { .. }) => false,
            Self::Type(_) => true,
            Self::Module(_) => true,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Variable(variable) => write!(f, "{:?}", variable),
            Self::Constant(constant) => write!(f, "{}", constant),
            Self::Static(r#static) => write!(f, "{:?}", r#static),
            Self::Type(r#type) => write!(f, "{}", r#type),
            Self::Module(_) => write!(f, "<module>"),
        }
    }
}