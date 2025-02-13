#[derive(Clone, Copy)]
pub(crate) enum NumType {
    I32,
    I64,
    F32,
    F64,
}

// TODO: VecType.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RefType {
    FuncRef,
    ExternRef,
}

pub(crate) enum ValType {
    NumType(NumType),
    // TODO: VecType
    RefType(RefType),
}

pub(crate) struct ResultType {
    types: Vec<ValType>,
}

pub(crate) struct FuncType {
    input: ResultType,
    output: ResultType,
}

pub(crate) struct Limits {
    min: u32,
    max: Option<u32>,
}

pub(crate) struct MemType {
    limits: Limits,
}

pub(crate) struct TableType {
    limits: Limits,
    type_: RefType,
}

pub(crate) struct GlobalType {
    mut_: Mut,
    val_type: ValType,
}

pub(crate) enum Mut {
    Const,
    Var,
}

pub(crate) enum ExternalType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
