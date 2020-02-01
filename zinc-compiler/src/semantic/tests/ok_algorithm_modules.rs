//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use zinc_bytecode::Add;
use zinc_bytecode::Call;
use zinc_bytecode::Cast;
use zinc_bytecode::Exit;
use zinc_bytecode::Instruction;
use zinc_bytecode::Load;
use zinc_bytecode::LoadGlobal;
use zinc_bytecode::Mul;
use zinc_bytecode::PushConst;
use zinc_bytecode::Return;
use zinc_bytecode::Store;
use zinc_bytecode::StoreGlobal;
use zinc_bytecode::Sub;

use crate::Bytecode;
use crate::Scope;

static PANIC_COMPILE_DEPENDENCY: &str = "Dependencies are compiled successfullt";

#[test]
fn test() {
    let module_1 = r#"
fn sum(a: field, b: field) -> field {
    a + b
}
"#;

    let module_2 = r#"
fn diff(a: field, b: field) -> field {
    a - b
}
"#;

    let module_3 = r#"
fn factor(a: field, b: field) -> field {
    a * b
}
"#;

    let binary = r#"
mod module_1;
mod module_2;
mod module_3;

static STATIC: field = 5;

const CONST: field = 42;

fn main(input: (), witness: ()) -> field {
    let var: field = 69;

    module_3::factor(module_2::diff(module_1::sum(STATIC, CONST), var), 5 as field)
}
"#;

    let expected = Ok(vec![
        Instruction::Call(Call::new(17, 0)),
        Instruction::Exit(Exit::new(1)),
        Instruction::Load(Load::new(1)),
        Instruction::Load(Load::new(0)),
        Instruction::Add(Add),
        Instruction::Return(Return::new(1)),
        Instruction::Load(Load::new(1)),
        Instruction::Load(Load::new(0)),
        Instruction::Sub(Sub),
        Instruction::Return(Return::new(1)),
        Instruction::Load(Load::new(1)),
        Instruction::Load(Load::new(0)),
        Instruction::Mul(Mul),
        Instruction::Return(Return::new(1)),
        Instruction::PushConst(PushConst::new(
            BigInt::from(5),
            false,
            crate::BITLENGTH_BYTE,
        )),
        Instruction::Cast(Cast::new(false, crate::BITLENGTH_FIELD)),
        Instruction::StoreGlobal(StoreGlobal::new(0)),
        Instruction::PushConst(PushConst::new(
            BigInt::from(69),
            false,
            crate::BITLENGTH_BYTE,
        )),
        Instruction::Cast(Cast::new(false, crate::BITLENGTH_FIELD)),
        Instruction::Store(Store::new(0)),
        Instruction::LoadGlobal(LoadGlobal::new(0)),
        Instruction::PushConst(PushConst::new(
            BigInt::from(42),
            false,
            crate::BITLENGTH_FIELD,
        )),
        Instruction::Call(Call::new(2, 2)),
        Instruction::Load(Load::new(0)),
        Instruction::Call(Call::new(6, 2)),
        Instruction::PushConst(PushConst::new(
            BigInt::from(5),
            false,
            crate::BITLENGTH_BYTE,
        )),
        Instruction::Cast(Cast::new(false, crate::BITLENGTH_FIELD)),
        Instruction::Call(Call::new(10, 2)),
        Instruction::Return(Return::new(1)),
    ]);

    let bytecode = Rc::new(RefCell::new(Bytecode::new()));
    let module_1 =
        super::get_dependency(module_1, bytecode.clone()).expect(PANIC_COMPILE_DEPENDENCY);
    let module_2 =
        super::get_dependency(module_2, bytecode.clone()).expect(PANIC_COMPILE_DEPENDENCY);
    let module_3 =
        super::get_dependency(module_3, bytecode.clone()).expect(PANIC_COMPILE_DEPENDENCY);

    let dependencies: HashMap<String, Rc<RefCell<Scope>>> = vec![
        ("module_1".to_owned(), module_1),
        ("module_2".to_owned(), module_2),
        ("module_3".to_owned(), module_3),
    ]
    .into_iter()
    .collect();

    let result = super::get_instructions_with_dependencies(binary, bytecode, dependencies);

    assert_eq!(expected, result);
}