#![allow(unused)]

use std::{
    array,
    cmp::PartialOrd,
    convert::{From, TryFrom},
    fmt::Debug,
    iter,
    mem::take,
    ops::{Add, Sub},
};

use wasmbin::{
    indices::{FuncId, LocalId},
    instructions::{self, Instruction},
    sections::{FuncBody, Locals},
    types::{BlockType, ValueType},
};

use crate::{
    store::Store,
    values::{self, Val},
};

pub type Result<'store, T = ComputationStatus<'store>, E = Trap> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum EvaluationStatus {
    Value(Vec<Val>),
    // Interrupted
}

pub enum ComputationStatus<'store> {
    ContinueToNext,
    Call(&'store FuncBody),
    ContinueToElse,
    ContinueToEnd,
    Return,
    // TODO: interruption.
}

#[derive(Debug)]
pub struct Trap;

pub struct Interpreter<'store> {
    pub instr_stack: Vec<(usize, &'store FuncBody)>,
    runner: InstructionRunner<'store>,
}

#[derive(Debug)]
struct InstructionRunner<'store> {
    stack: Vec<Val>,
    locals: Vec<Frame>,
    pub store: &'store Store,
}

impl Interpreter<'_> {
    /// Creates a new [`Interpreter`].
    pub fn new(store: &mut Store) -> Interpreter<'_> {
        Interpreter {
            instr_stack: Vec::new(),
            runner: InstructionRunner::new(store),
        }
    }

    pub fn run(
        &mut self,
        func_name: &str,
        args: impl IntoIterator<Item = Val>,
    ) -> Result<EvaluationStatus, Trap> {
        let (func_type, func_body) = self.runner.store.find_function(func_name);
        let func_type = &self.runner.store[*func_type];

        let args = args.into_iter().collect::<Vec<_>>();

        assert_eq!(func_type.params.len(), args.len());

        self.instr_stack.push((0, func_body));

        self.runner
            .push_frame(args, &func_body.locals, &func_type.results);

        loop {
            let (cursor, func) = match self.instr_stack.last_mut() {
                Some((cursor, func)) => (cursor, func as &FuncBody),
                None => return Ok(EvaluationStatus::Value(take(&mut self.runner.stack))),
            };

            if *cursor >= func.expr.len() {
                self.runner.return_from_func();
                self.instr_stack.pop();
                continue;
            }

            let status = match &func.expr[*cursor] {
                Instruction::LocalGet(local) => self.runner.local_get(local),
                Instruction::I32Add => self.runner.add::<i32>(),
                Instruction::Call(func) => self.runner.call(*func),
                Instruction::I32Const(val) => self.runner.const_::<i32>(*val),
                Instruction::I32LtU => self.runner.lt_u::<i32>(),
                Instruction::IfStart(block_type) => self.runner.if_start(block_type),
                Instruction::IfElse => self.runner.if_else(),
                Instruction::I32Sub => self.runner.sub::<i32>(),
                Instruction::End => self.runner.end(),
                Instruction::LocalSet(local) => self.runner.local_set(*local),
                Instruction::Return => self.runner.return_(),

                unknown => unreachable!("unknown instruction: `{unknown:?}`"),
            }?;

            *cursor += 1;
            match status {
                ComputationStatus::ContinueToNext => {}

                ComputationStatus::Call(func) => self.instr_stack.push((0, func)),

                ComputationStatus::ContinueToElse => {
                    *cursor = Self::continue_to(func, Instruction::IfElse, *cursor);
                }

                ComputationStatus::ContinueToEnd => {
                    *cursor = Self::continue_to(func, Instruction::End, *cursor);
                }

                ComputationStatus::Return => {
                    self.instr_stack.pop().unwrap();
                }
            }
        }
    }

    fn continue_to(func: &FuncBody, instr: Instruction, mut cursor: usize) -> usize {
        let mut depth = 0;

        loop {
            match &func.expr[cursor] {
                instr_ if depth == 0 && instr_ == &instr => {
                    break cursor + 1;
                }

                Instruction::BlockStart(_) | Instruction::IfStart(_) => depth += 1,

                Instruction::End => depth -= 1,

                _ => {}
            }

            cursor += 1;
        }
    }
}

impl<'store> InstructionRunner<'store> {
    fn new(store: &'store mut Store) -> InstructionRunner<'store> {
        Self {
            locals: Vec::new(),
            stack: Vec::new(),
            store,
        }
    }

    fn push_frame(
        &mut self,
        arguments: impl IntoIterator<Item = Val>,
        other_locals: &[Locals],
        result: &[ValueType],
    ) {
        let locals = arguments
            .into_iter()
            .chain(other_locals.iter().flat_map(|Locals { repeat, ty }| {
                iter::repeat_n(Val::default(ty), *repeat as usize)
            }))
            .collect();

        let frame = Frame {
            init_stack_size: self.stack.len(),
            locals,
            arity: result.len(),
        };

        self.locals.push(frame);
    }

    fn pop_frame(&mut self) {
        self.locals.pop().unwrap();
    }

    fn local_get(&mut self, local: &LocalId) -> Result<'store> {
        let val = self.local_value(*local);
        self.stack.push(val);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn run_const(&mut self, val: Val) -> Result<'store> {
        self.stack.push(val);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn add<T>(&mut self) -> Result<'store>
    where
        T: TryFrom<Val> + Add,
        <T as TryFrom<Val>>::Error: Debug,
        Val: From<<T as std::ops::Add>::Output>,
    {
        self.binop::<T>(|l, r| Some(Val::from(l + r)))
    }

    fn call(&mut self, func: FuncId) -> Result<'store> {
        let (type_, function) = &self.store[func];
        let func_type = &self.store[*type_];
        let locals = self
            .stack
            .drain(self.stack.len() - func_type.params.len()..)
            .collect();

        let init_stack_size = self.stack.len();

        let frame = Frame {
            locals,
            init_stack_size,
            arity: func_type.results.len(),
        };

        self.locals.push(frame);

        Ok(ComputationStatus::Call(function))
    }

    fn const_<T>(&mut self, val: T) -> Result<'store>
    where
        Val: From<T>,
    {
        self.stack.push(val.into());

        Ok(ComputationStatus::ContinueToNext)
    }

    fn lt_u<T>(&mut self) -> Result<'store>
    where
        T: TryFrom<Val> + PartialOrd,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.binop::<T>(|lhs, rhs| {
            let v = if lhs < rhs { 1 } else { 0 };
            let v = v.into();
            Some(v)
        })
    }

    fn if_start(&mut self, block_type: &BlockType) -> Result<'store> {
        match self.pop::<i32>() {
            0 => Ok(ComputationStatus::ContinueToElse),
            // The then branch is the next instruction.
            _ => Ok(ComputationStatus::ContinueToNext),
        }
    }

    fn if_else(&self) -> Result<'store> {
        Ok(ComputationStatus::ContinueToEnd)
    }

    fn sub<T>(&mut self) -> Result<'store>
    where
        T: TryFrom<values::Val> + Sub<Output = T>,
        <T as std::convert::TryFrom<values::Val>>::Error: std::fmt::Debug,
        Val: From<T>,
    {
        self.binop::<T>(|lhs, rhs| Some(Val::from(lhs - rhs)))
    }

    fn end(&self) -> Result<'store> {
        Ok(ComputationStatus::ContinueToNext)
    }

    fn local_set(&mut self, local: LocalId) -> Result<'store> {
        let top = self.stack.pop().unwrap();

        *self
            .locals
            .last_mut()
            .unwrap()
            .locals
            .get_mut(local.index as usize)
            .unwrap() = top;

        Ok(ComputationStatus::ContinueToNext)
    }

    fn return_(&mut self) -> Result<'store> {
        let old_frame = self.locals.pop().unwrap();
        let result_last = self.stack.len();
        let result_first = result_last - old_frame.arity;
        let result = self
            .stack
            .drain(result_first..result_last)
            .collect::<Vec<_>>();

        // Let's just check that the maths are mathing.
        debug_assert_eq!(result.len(), old_frame.arity);

        self.stack.truncate(old_frame.init_stack_size);
        self.stack.extend(result);

        Ok(ComputationStatus::Return)
    }

    fn unop<T>(&mut self, f: impl FnOnce(T) -> Option<Val>) -> Result<'store>
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.apply_typed::<T, 1>(|[a]| f(a))
    }

    fn binop<T>(&mut self, f: impl FnOnce(T, T) -> Option<Val>) -> Result<'store>
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.apply_typed::<T, 2>(|[a, b]| f(a, b))
    }

    fn testop<T>(&mut self, f: impl FnOnce(T) -> Val) -> Result<'store>
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.unop(|val| Some(f(val)))
    }

    fn apply_typed<T, const N: usize>(
        &mut self,
        f: impl FnOnce([T; N]) -> Option<Val>,
    ) -> Result<'store>
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        let mut args = array::from_fn(|_| self.stack.pop().unwrap().try_into().unwrap());
        args.reverse();

        let out = f(args).ok_or(Trap)?;

        self.stack.push(out);

        Ok(ComputationStatus::ContinueToNext)
    }

    fn local_value(&self, local: LocalId) -> Val {
        *self
            .locals
            .last()
            .unwrap()
            .locals
            .get(local.index as usize)
            .unwrap()
    }

    fn return_from_func(&mut self) {
        self.locals.pop().unwrap();
    }

    fn pop<T>(&mut self) -> T
    where
        T: TryFrom<Val>,
        <T as TryFrom<Val>>::Error: Debug,
    {
        self.stack.pop().unwrap().try_into().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Frame {
    // The length of the stack when the frame is created
    init_stack_size: usize,
    locals: Vec<Val>,
    arity: usize,
}
