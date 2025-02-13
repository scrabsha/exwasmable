pub(crate) fn parse_all(program: &[u8]) -> wasmbin::Module {
    wasmbin::Module::decode_from(program).unwrap()
}
