use std::{iter::zip, mem::take, ops::Index};

use wasmbin::{
    indices::{FuncId, TypeId},
    sections::{
        self, ExportDesc, FuncBody,
        payload::{Code, Export, Function, Type},
    },
    types::FuncType,
};

/// Aggregates the data from multiple WASM modules.
///
/// Once this struct is fully populated, it is passed to the interpreter.
// Data stored here must be in a format that is ready to use by the interpreter.
#[derive(Debug)]
pub struct Store {
    funcs: Vec<(TypeId, FuncBody)>,
    exports: Vec<sections::Export>,
    types: Vec<FuncType>,
}

impl Store {
    pub(crate) fn new(mut module: wasmbin::Module) -> Self {
        let function_section = module.find_std_section_mut::<Function>().unwrap();
        let function_section = function_section
            .try_contents_mut()
            .unwrap()
            .iter_mut()
            .map(|func_type| *func_type)
            .collect::<Vec<_>>();

        let code_section = module.find_std_section_mut::<Code>().unwrap();
        let code_section = code_section
            .try_contents_mut()
            .unwrap()
            .iter_mut()
            .map(|body| take(body.try_contents_mut().unwrap()));

        let funcs = zip(function_section, code_section).collect();

        let exports = module
            .find_std_section_mut::<Export>()
            .unwrap()
            .try_contents_mut()
            .map(take)
            .unwrap();

        let type_section = module.find_std_section_mut::<Type>().unwrap();

        let types = type_section.try_contents_mut().unwrap().drain(..).collect();

        Store {
            funcs,
            exports,
            types,
        }
    }

    pub(crate) fn find_function(&self, sym_name: &str) -> &(TypeId, FuncBody) {
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

        self.funcs.get(f.index as usize).unwrap()
    }
}

impl Index<FuncId> for Store {
    type Output = (TypeId, FuncBody);
    fn index(&self, func: FuncId) -> &Self::Output {
        &self.funcs[func.index as usize]
    }
}

impl Index<TypeId> for Store {
    type Output = FuncType;

    fn index(&self, type_: TypeId) -> &Self::Output {
        &self.types[type_.index as usize]
    }
}
