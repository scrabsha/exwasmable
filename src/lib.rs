pub mod interpreter;
pub mod parser;
pub mod store;
mod types;
mod values;

#[cfg(test)]
mod tests {
    use interpreter::{EvaluationStatus, Interpreter};
    use store::Store;

    use super::*;

    macro_rules! inline_wasm {
        ( $( $code:tt )* ) => {
            {
                let program = concat!($( stringify!($code), " ",)*);
                wat::parse_str(program).unwrap()
            }
        };
    }

    #[test]
    fn adder_simple() {
        let program = inline_wasm! {
            (module
              (func $add (param $lhs i32) (param $rhs i32) (result i32)
                local.get $lhs
                local.get $rhs
                i32.add)
              (export "add" (func $add))
            )
        };

        let module = parser::parse_all(&program);

        let mut store = Store::new(module);
        let mut interpreter = Interpreter::new(&mut store);

        let values = interpreter
            .run("add", [41_i32.into(), 1_i32.into()])
            .unwrap();

        assert_eq!(values, EvaluationStatus::Value(vec![42_i32.into()]));
    }

    #[test]
    fn function_call() {
        let program = inline_wasm! {
            (module
              (func $add_inner (param $lhs i32) (param $rhs i32) (result i32)
                local.get $lhs
                local.get $rhs
                i32.add)
              (func $add (param $lhs i32) (param $rhs i32) (result i32)
                local.get $lhs
                local.get $rhs
                call $add_inner)
              (export "add" (func $add))
            )
        };

        let module = parser::parse_all(&program);

        let mut store = Store::new(module);

        let mut interpreter = Interpreter::new(&mut store);

        let values = interpreter
            .run("add", [41_i32.into(), 1_i32.into()])
            .unwrap();

        assert_eq!(values, EvaluationStatus::Value(vec![42i32.into()]));
    }

    #[test]
    fn recursive_fib() {
        #[rustfmt::skip]
        let program = inline_wasm! {
            (module
                (func $fib (param $n i32) (result i32)
                    local.get $n
                    i32.const 2
                    i32.lt_u
                    if (result i32)
                        (local.get $n)
                      else
                        (local.get $n)
                        (i32.const 1)
                        (i32.sub)
                        (call $fib)
                        (local.get $n)
                        (i32.const 2)
                        (i32.sub)
                        (call $fib)
                        (i32.add)
                  end)
            (export "fib" (func $fib)))
        };

        let module = parser::parse_all(&program);

        let mut store = Store::new(module);

        let mut interpreter = Interpreter::new(&mut store);

        let values = interpreter.run("fib", [10_i32.into()]).unwrap();

        assert_eq!(values, EvaluationStatus::Value(vec![55_i32.into()]));
    }
}

#[cfg(test)]
mod rust {
    use crate::{
        interpreter::{EvaluationStatus, Interpreter},
        parser,
        store::Store,
        values::v,
    };

    macro_rules! wasm {
    ($( $code:tt )* ) => {{
        let code = concat!($( stringify!($code), " ",)*);

        let mut children = ::std::process::Command::new("rustc")
            .args(["-C", "panic=abort"])
            .args(["--target", "wasm32-unknown-unknown"])
            .arg("-")
            .args(["-o", "-"])
            .args(["--crate-type", "cdylib"])
            .stdin(::std::process::Stdio::piped())
            .stdout(::std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = children.stdin.take().unwrap();
        ::std::io::Write::write_all(&mut stdin, code.as_bytes()).unwrap();
        drop(stdin);

        let mut bytes = Vec::new();
        let mut stdout = children.stdout.take().unwrap();
        ::std::io::Read::read_to_end(&mut stdout, &mut bytes).unwrap();
        drop(stdout);

        children.wait().unwrap();

        bytes
    }};
}

    #[test]
    fn add_simple() {
        let bytecode = wasm! {
            #[no_mangle]
            pub fn add(a: u32, b: u32) -> u32 {
                a.wrapping_add(b)
            }

        };

        let module = parser::parse_all(&bytecode);

        let mut store = Store::new(module);

        let mut interpreter = Interpreter::new(&mut store);

        let values = interpreter.run("add", [v(41_i32), v(1_i32)]).unwrap();

        assert_eq!(values, EvaluationStatus::Value(vec![v(42_i32)]));
    }
}
