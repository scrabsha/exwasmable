use wasmbin::sections::{
    self, ExportDesc, FuncBody,
    payload::{Code, Export},
};

/// Aggregates the data from multiple WASM modules.
///
/// Once this struct is fully populated, it is passed to the interpreter.
// Data stored here must be in a format that is ready to use by the interpreter.
pub struct Store {
    module: wasmbin::Module,
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
    pub(crate) fn new(module: wasmbin::Module) -> Self {
        Self {
            module,
            // funcs: todo!(),
            // tables: todo!(),
            // mems: todo!(),
            // globals: todo!(),
            // elems: todo!(),
            // datas: todo!(),
        }
    }

    pub(crate) fn find_function(&self, sym_name: &str) -> FuncBody {
        let export_section = self.exports();

        let desc = export_section
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

        let code_section = self.code();

        code_section.get(f.index as usize).unwrap().clone()
    }

    fn exports(&self) -> &[sections::Export] {
        self.module
            .find_std_section::<Export>()
            .unwrap()
            .try_contents()
            .unwrap()
            .as_slice()
    }

    fn code(&self) -> Vec<FuncBody> {
        // TODO: this is a very hot path, as we constantly decode every function.
        self.module
            .find_std_section::<Code>()
            .unwrap()
            .try_contents()
            .unwrap()
            .iter()
            .map(|blob| blob.contents.clone().try_into_contents().unwrap())
            .collect()
    }
}
