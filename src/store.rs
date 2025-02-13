use std::mem::take;

use wasmbin::sections::{
    self, ExportDesc, FuncBody,
    payload::{Code, Export, Function},
};

/// Aggregates the data from multiple WASM modules.
///
/// Once this struct is fully populated, it is passed to the interpreter.
// Data stored here must be in a format that is ready to use by the interpreter.
pub struct Store {
    funcs: Vec<FuncBody>,
    exports: Vec<sections::Export>,
    // funcs: Vec<FuncInst>,
    // tables: Vec<TableInst>,
    // mems: Vec<MemInst>,
    // globals: Vec<GlobalInst>,
    // elems: Vec<ElemInst>,
    // datas: Vec<DataInst>,
}

// TODO: support host functions.
// pub(crate) struct FuncInst {
//     type_: FuncType,
//     module: ModuleInst,
// TODO: this _will_ need some conversion in the future.
// code: Func,
// }

// pub(crate) struct TableInst {
// type_: TableType,
// elem: Vec<Ref>,
// }

struct MemInst;

struct GlobalInst;

struct ElemInst;

struct DataInst;

struct ModuleInst;

impl Store {
    pub(crate) fn new(mut module: wasmbin::Module) -> Self {
        let code_section = module.find_std_section_mut::<Code>().unwrap();

        let funcs = code_section
            .try_contents_mut()
            .unwrap()
            .iter_mut()
            .map(|body| body.try_contents_mut().unwrap())
            .map(take)
            .collect();

        let exports = module
            .find_std_section_mut::<Export>()
            .unwrap()
            .try_contents_mut()
            .map(take)
            .unwrap();

        Self {
            funcs,
            exports,
            // funcs: todo!(),
            // tables: todo!(),
            // mems: todo!(),
            // globals: todo!(),
            // elems: todo!(),
            // datas: todo!(),
        }
    }

    pub(crate) fn find_function(&self, sym_name: &str) -> FuncBody {
        let desc = self
            .exports
            .iter()
            .find_map(|export| {
                if export.name == sym_name {
                    Some(&export.desc)
                } else {
                    None
                }
            })
            .unwrap();

        let ExportDesc::Func(f) = desc else { panic!() };

        self.funcs.get(f.index as usize).unwrap().clone()
    }
}
