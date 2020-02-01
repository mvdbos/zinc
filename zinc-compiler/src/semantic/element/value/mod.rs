//!
//! The semantic analyzer value element.
//!

mod array;
mod error;
mod integer;
mod structure;
mod tuple;

pub use self::array::Array;
pub use self::array::Error as ArrayError;
pub use self::error::Error;
pub use self::integer::Error as IntegerError;
pub use self::integer::Integer;
pub use self::structure::Error as StructureError;
pub use self::structure::Structure;
pub use self::tuple::Error as TupleError;
pub use self::tuple::Tuple;

use std::fmt;

use crate::semantic::Caster;
use crate::semantic::Constant;
use crate::semantic::FieldAccessResult;
use crate::semantic::IndexAccessResult;
use crate::semantic::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Boolean,
    Integer(Integer),
    Array(Array),
    Tuple(Tuple),
    Structure(Structure),
}

impl Value {
    pub fn new(r#type: Type) -> Self {
        match r#type {
            Type::Unit => Self::Unit,
            Type::Boolean => Self::Boolean,
            Type::IntegerUnsigned { bitlength } => Self::Integer(Integer::new(false, bitlength)),
            Type::IntegerSigned { bitlength } => Self::Integer(Integer::new(true, bitlength)),
            Type::Field => Self::Integer(Integer::new(false, crate::BITLENGTH_FIELD)),
            Type::Array { r#type, size } => Self::Array(Array::new(*r#type, size)),
            Type::Tuple { types } => Self::Tuple(Tuple::new(types)),
            Type::Structure {
                identifier, fields, ..
            } => Self::Structure(Structure::new(identifier, fields)),
            Type::Enumeration { bitlength, .. } => Self::Integer(Integer::new(false, bitlength)),
            r#type => panic!(
                "{}{}",
                crate::semantic::PANIC_VALUE_CANNOT_BE_CREATED_FROM,
                r#type
            ),
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Self::Unit => Type::new_unit(),
            Self::Boolean => Type::new_boolean(),
            Self::Integer(integer) => integer.r#type(),
            Self::Array(array) => array.r#type(),
            Self::Tuple(tuple) => tuple.r#type(),
            Self::Structure(structure) => structure.r#type(),
        }
    }

    pub fn has_the_same_type_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Boolean, Self::Boolean) => true,
            (Self::Integer(value_1), Self::Integer(value_2)) => {
                value_1.has_the_same_type_as(value_2)
            }
            (Self::Array(value_1), Self::Array(value_2)) => value_1.has_the_same_type_as(value_2),
            (Self::Tuple(value_1), Self::Tuple(value_2)) => value_1.has_the_same_type_as(value_2),
            (Self::Structure(value_1), Self::Structure(value_2)) => {
                value_1.has_the_same_type_as(value_2)
            }
            _ => false,
        }
    }

    pub fn or(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Boolean => match other {
                Self::Boolean => Ok(Self::Boolean),
                value => Err(Error::OperatorOrSecondOperandExpectedBoolean(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorOrFirstOperandExpectedBoolean(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn xor(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Boolean => match other {
                Self::Boolean => Ok(Self::Boolean),
                value => Err(Error::OperatorXorSecondOperandExpectedBoolean(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorXorFirstOperandExpectedBoolean(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn and(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Boolean => match other {
                Self::Boolean => Ok(Self::Boolean),
                value => Err(Error::OperatorAndSecondOperandExpectedBoolean(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorAndFirstOperandExpectedBoolean(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn equals(&self, other: &Self) -> Result<Self, Error> {
        match (self, other) {
            (Self::Unit, Self::Unit) => Ok(Self::Boolean),
            (Self::Unit, value_2) => Err(Error::OperatorEqualsSecondOperandExpectedUnit(
                value_2.r#type().to_string(),
            )),
            (Self::Boolean, Self::Boolean) => Ok(Self::Boolean),
            (Self::Boolean, value_2) => Err(Error::OperatorEqualsSecondOperandExpectedBoolean(
                value_2.r#type().to_string(),
            )),
            (Self::Integer(integer_1), Self::Integer(integer_2)) => integer_1
                .equals(integer_2)
                .map(|_| Self::Boolean)
                .map_err(Error::Integer),
            (Self::Integer(_), value_2) => Err(Error::OperatorEqualsSecondOperandExpectedInteger(
                value_2.r#type().to_string(),
            )),
            (value_1, _) => Err(Error::OperatorEqualsFirstOperandExpectedPrimitiveType(
                value_1.r#type().to_string(),
            )),
        }
    }

    pub fn not_equals(&self, other: &Self) -> Result<Self, Error> {
        match (self, other) {
            (Self::Unit, Self::Unit) => Ok(Self::Boolean),
            (Self::Unit, value_2) => Err(Error::OperatorNotEqualsSecondOperandExpectedUnit(
                value_2.r#type().to_string(),
            )),
            (Self::Boolean, Self::Boolean) => Ok(Self::Boolean),
            (Self::Boolean, value_2) => Err(Error::OperatorNotEqualsSecondOperandExpectedBoolean(
                value_2.r#type().to_string(),
            )),
            (Self::Integer(integer_1), Self::Integer(integer_2)) => integer_1
                .not_equals(integer_2)
                .map(|_| Self::Boolean)
                .map_err(Error::Integer),
            (Self::Integer(_), value_2) => Err(
                Error::OperatorNotEqualsSecondOperandExpectedInteger(value_2.r#type().to_string()),
            ),
            (value_1, _) => Err(Error::OperatorNotEqualsFirstOperandExpectedPrimitiveType(
                value_1.r#type().to_string(),
            )),
        }
    }

    pub fn greater_equals(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .greater_equals(integer_2)
                    .map(|_| Self::Boolean)
                    .map_err(Error::Integer),
                value => Err(Error::OperatorGreaterEqualsSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorGreaterEqualsFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn lesser_equals(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .lesser_equals(integer_2)
                    .map(|_| Self::Boolean)
                    .map_err(Error::Integer),
                value => Err(Error::OperatorLesserEqualsSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorLesserEqualsFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn greater(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .greater(integer_2)
                    .map(|_| Self::Boolean)
                    .map_err(Error::Integer),
                value => Err(Error::OperatorGreaterSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorGreaterFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn lesser(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .lesser(integer_2)
                    .map(|_| Self::Boolean)
                    .map_err(Error::Integer),
                value => Err(Error::OperatorLesserSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorLesserFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .add(integer_2)
                    .map(|_| Self::Integer(integer_1.to_owned()))
                    .map_err(Error::Integer),
                value => Err(Error::OperatorAdditionSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorAdditionFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .subtract(integer_2)
                    .map(|_| Self::Integer(integer_1.to_owned()))
                    .map_err(Error::Integer),
                value => Err(Error::OperatorSubtractionSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorSubtractionFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn multiply(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .multiply(integer_2)
                    .map(|_| Self::Integer(integer_1.to_owned()))
                    .map_err(Error::Integer),
                value => Err(Error::OperatorMultiplicationSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorMultiplicationFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn divide(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .divide(integer_2)
                    .map(|_| Self::Integer(integer_1.to_owned()))
                    .map_err(Error::Integer),
                value => Err(Error::OperatorDivisionSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorDivisionFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn remainder(&self, other: &Self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer_1) => match other {
                Self::Integer(integer_2) => integer_1
                    .remainder(integer_2)
                    .map(|_| Self::Integer(integer_1.to_owned()))
                    .map_err(Error::Integer),
                value => Err(Error::OperatorRemainderSecondOperandExpectedInteger(
                    value.r#type().to_string(),
                )),
            },
            value => Err(Error::OperatorRemainderFirstOperandExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn cast(&mut self, to: &Type) -> Result<Option<(bool, usize)>, Error> {
        let from = self.r#type();
        Caster::validate(&from, &to).map_err(Error::Casting)?;

        let (is_signed, bitlength) = match to {
            Type::IntegerUnsigned { bitlength } => (false, *bitlength),
            Type::IntegerSigned { bitlength } => (true, *bitlength),
            Type::Field => (false, crate::BITLENGTH_FIELD),
            _ => return Ok(None),
        };

        if let Self::Integer(integer) = self {
            integer.cast(is_signed, bitlength).map_err(Error::Integer)?;
        }
        Ok(Some((is_signed, bitlength)))
    }

    pub fn negate(&self) -> Result<Self, Error> {
        match self {
            Self::Integer(integer) => integer
                .negate()
                .map(|_| Self::Integer(integer.to_owned()))
                .map_err(Error::Integer),
            value => Err(Error::OperatorNegationExpectedInteger(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn not(&self) -> Result<Self, Error> {
        match self {
            Self::Boolean => Ok(Self::Boolean),
            value => Err(Error::OperatorNotExpectedBoolean(
                value.r#type().to_string(),
            )),
        }
    }

    pub fn index_value(&self, other: &Self) -> Result<IndexAccessResult, Error> {
        match self {
            Value::Array(array) => match other {
                Value::Integer(_) => Ok(array.slice_single()),
                value => Err(Error::OperatorIndexSecondOperandExpectedIntegerOrRange(
                    value.to_string(),
                )),
            },
            value => Err(Error::OperatorIndexFirstOperandExpectedArray(
                value.to_string(),
            )),
        }
    }

    pub fn index_constant(&self, other: &Constant) -> Result<IndexAccessResult, Error> {
        match self {
            Value::Array(array) => match other {
                Constant::Integer(_) => Ok(array.slice_single()),
                Constant::Range(range) => array
                    .slice_range(&range.start, &range.end)
                    .map_err(Error::Array),
                Constant::RangeInclusive(range) => array
                    .slice_range_inclusive(&range.start, &range.end)
                    .map_err(Error::Array),
                constant => Err(Error::OperatorIndexSecondOperandExpectedIntegerOrRange(
                    constant.to_string(),
                )),
            },
            value => Err(Error::OperatorIndexFirstOperandExpectedArray(
                value.to_string(),
            )),
        }
    }

    pub fn field_tuple(&self, field_index: usize) -> Result<FieldAccessResult, Error> {
        match self {
            Value::Tuple(tuple) => tuple.slice(field_index).map_err(Error::Tuple),
            value => Err(Error::OperatorFieldFirstOperandExpectedTuple(
                value.to_string(),
            )),
        }
    }

    pub fn field_structure(&self, field_name: &str) -> Result<FieldAccessResult, Error> {
        match self {
            Value::Structure(structure) => structure.slice(field_name).map_err(Error::Structure),
            value => Err(Error::OperatorFieldFirstOperandExpectedStructure(
                value.to_string(),
            )),
        }
    }
}

impl From<Constant> for Value {
    fn from(constant: Constant) -> Self {
        Self::new(constant.r#type())
    }
}

impl From<&Constant> for Value {
    fn from(constant: &Constant) -> Self {
        Self::new(constant.r#type())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type())
    }
}