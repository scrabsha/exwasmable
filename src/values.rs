use crate::types::{FuncType, NumType, RefType, ValType};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ref {
    Null(RefType),
    Ref(FuncAddr),
    Extern(ExternAddr),
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Val {
    Num(Num),
    // TODO: vec
    Ref(Ref),
}

impl Val {
    pub fn default(val_type: ValType) -> Val {
        match val_type {
            ValType::NumType(NumType::I32) => Val::i32(0),
            ValType::NumType(NumType::I64) => Val::i64(0),
            ValType::NumType(NumType::F32) => Val::f32(0.0),
            ValType::NumType(NumType::F64) => Val::f64(0.0),
            ValType::RefType(ty) => Val::null(ty),
        }
    }

    pub(crate) fn i32(i: i32) -> Val {
        Val::Num(Num::I32(i))
    }

    pub(crate) fn i64(i: i64) -> Val {
        Val::Num(Num::I64(i))
    }

    pub(crate) fn f32(f: f32) -> Val {
        Val::Num(Num::F32(f))
    }

    pub(crate) fn f64(f: f64) -> Val {
        Val::Num(Num::F64(f))
    }

    fn null(ty: RefType) -> Val {
        Val::Ref(Ref::Null(ty))
    }

    fn ref_(addr: FuncAddr) -> Val {
        Val::Ref(Ref::Ref(addr))
    }

    fn extern_(addr: ExternAddr) -> Val {
        Val::Ref(Ref::Extern(addr))
    }
}

impl Default for Val {
    fn default() -> Self {
        Val::i32(0)
    }
}

macro_rules! impl_try_from_and_into {
    ($ty:ty, ( $( $lhs:tt )* ) <=> ( $( $rhs:tt )* )) => {
        impl TryFrom<Val> for $ty {
            type Error = ();

            fn try_from(value: Val) -> Result<Self, ()> {
                match value {
                    $( $lhs )* => Ok($( $rhs )*),
                    _ => Err(()),
                }
            }
        }

        impl From<$ty> for Val {
            fn from($( $rhs )*: $ty) -> Val {
                $( $lhs )*
            }
        }
    };
}

impl_try_from_and_into!(i32, (Val::Num(Num::I32(val))) <=> (val));
impl_try_from_and_into!(i64, (Val::Num(Num::I64(val))) <=> (val));
impl_try_from_and_into!(f32, (Val::Num(Num::F32(val))) <=> (val));
impl_try_from_and_into!(f64, (Val::Num(Num::F64(val))) <=> (val));

enum WasmResult {
    Value(Vec<Val>),
    Trap,
}

struct Store {
    funcs: Vec<FuncInst>,
    tables: Vec<TableInst>,
    mems: Vec<MemInst>,
    globals: Vec<GlobalInst>,
    elems: Vec<ElemInst>,
    datas: Vec<DataInst>,
}

struct ModuleInst {}

macro_rules! addr_ty {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub u32);
    };
}

addr_ty!(Addr);
addr_ty!(FuncAddr);
addr_ty!(TableAddr);
addr_ty!(MemAddr);
addr_ty!(GlobalAddr);
addr_ty!(ElemAddr);
addr_ty!(DataAddr);
addr_ty!(ExternAddr);

enum FuncInst {
    Local { type_: FuncType, module: ModuleInst },
}

struct TableInst;

struct MemInst;

struct GlobalInst;

struct ElemInst;

struct DataInst;
