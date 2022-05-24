//!
//! The semantic analyzer structure type element.
//!

mod tests;

pub mod error;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::semantic::element::r#type::Type;
use crate::semantic::scope::Scope;

///
/// Describes a structure type.
///
/// Consists of the local structure `identifier` within its scope, global `unique_id`, `fields`,
/// and the implementation `scope`, which contains the reference to its parent scope.
///
#[derive(Debug, Clone)]
pub struct Structure {
    pub identifier: String,
    pub unique_id: usize,
    pub fields: Vec<(String, Type)>,
    pub scope: Rc<RefCell<Scope>>,
}

// Implementation of intermodule dependencies for Zinc. 

// IMPORTANT NOTE ABOUT THE UPDATE
// Zinc version 0.1.5 does not allow to have the same method names within the scope of different
// structs. With this update, we add support for the same method name in separate structs, which
//  enables us to implement methods of similar nature for different structs. 

impl Structure {
    pub fn new(
        identifier: String,
        unique_id: usize,
        fields: Vec<(String, Type)>,
        scope_parent: Option<Rc<RefCell<Scope>>>,
    ) -> Self {
        let scope = Rc::new(RefCell::new(Scope::new(scope_parent)));

        let structure = Self {
            identifier,
            unique_id,
            fields,
            scope: scope.clone(),
        };

        scope
            .borrow_mut()
            .declare_self(Type::Structure(structure.clone()));

        structure
    }
}

impl PartialEq<Self> for Structure {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}

impl fmt::Display for Structure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "struct {}", self.identifier)
    }
}
