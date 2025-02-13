#![allow(unused)]

use std::{array, convert::TryFrom, fmt::Debug, mem::take, ops::Add};

use wasmbin::{
    indices::LocalId,
    instructions::{self, Instruction},
};

use crate::{
    store::Store,
    values::{self, Val},
};

pub type Result<T = ComputationStatus, E = Trap> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum EvaluationStatus {
    Value(Vec<Val>),
    // Interrupted
}

pub enum ComputationStatus {
    ContinueToNext,
    // TODO: interruption.
}

#[derive(Debug)]
pub struct Trap;

pub struct Interpreter<'store> {
    pub store: &'store Store,
    // ???
    pub local_stack: Vec<Vec<Val>>,
    pub value_stack: Vec<Val>,
    pub instr_stack: Vec<InstrPtr>,
}

impl Interpreter<'_> {
    /// Creates a new [`Interpreter`].
    pub fn new(store: &mut Store) -> Interpreter<'_> {
        Interpreter {
            local_stack: Vec::new(),
            value_stack: Vec::new(),
            instr_stack: Vec::new(),
            store,
        }
    }

    pub fn run(
        &mut self,
        func_name: &str,
        args: impl IntoIterator<Item = Val>,
    ) -> Result<EvaluationStatus, Trap> {
        let func_body = self.store.find_function(func_name);

        assert!(
            func_body.locals.is_empty(),
            "Not yet implemented: function locals"
        );

        let locals = args.into_iter().collect();
        self.local_stack.push(locals);

        let mut next_instr_idx = 0;

        loop {
            let status = match &func_body.expr[next_instr_idx] {
                Instruction::LocalGet(local) => self.run_local_get(local),
                Instruction::I32Add => self.run_add::<i32>(),

                unknown => unreachable!("unknown instruction: `{unknown:?}`"),
            }?;

            match status {
                ComputationStatus::ContinueToNext => {
                    next_instr_idx += 1;
                }
            }

            if next_instr_idx == func_body.expr.len() {
                break Ok(EvaluationStatus::Value(take(&mut self.value_stack)));
            }
        }
    }

    fn run_local_get(&mut self, local: &LocalId) -> Result {
        let val = self.local_value(*local);
        self.value_stack.push(val);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn run_const(&mut self, val: Val) -> Result {
        self.value_stack.push(val);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn run_add<T>(&mut self) -> Result
    where
        T: TryFrom<Val> + Add,
        <T as std::convert::TryFrom<values::Val>>::Error: std::fmt::Debug,
        values::Val: std::convert::From<<T as std::ops::Add>::Output>,
    {
        self.binop::<T>(|l, r| Some(Val::from(l + r)))
    }

    fn unop<T>(&mut self, f: impl FnOnce(T) -> Option<Val>) -> Result
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.apply_typed::<T, 1>(|[a]| f(a))
    }

    fn binop<T>(&mut self, f: impl FnOnce(T, T) -> Option<Val>) -> Result
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.apply_typed::<T, 2>(|[a, b]| f(a, b))
    }

    fn testop<T>(&mut self, f: impl FnOnce(T) -> Val) -> Result
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.unop(|val| Some(f(val)))
    }

    fn apply_typed<T, const N: usize>(&mut self, f: impl FnOnce([T; N]) -> Option<Val>) -> Result
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        let args = array::from_fn(|_| self.value_stack.pop().unwrap().try_into().unwrap());

        let out = f(args).ok_or(Trap)?;

        self.value_stack.push(out);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn local_value(&self, local: LocalId) -> Val {
        *self
            .local_stack
            .last()
            .unwrap()
            .get(local.index as usize)
            .unwrap()
    }
}

// TODO
type InstrPtr = u32;

/// Represents the content of a function.
struct FunctionInstructions {
    instrs: Vec<Instruction>,
}
