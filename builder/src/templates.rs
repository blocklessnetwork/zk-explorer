pub(crate) mod templates {
    macro_rules! WASM_BUILD_TEMPLATE_CARGO_TOML {
        () => {
            r#"[package]
name = "wasm-methods"
version = "0.1.0"
edition = "2021"

[build-dependencies]
risc0-build = { version = "0.17.0" }

[package.metadata.risc0]
methods = ["guest"]"#
        };
    }

    macro_rules! WASM_BUILD_TEMPLATE_LIB_RS {
        () => {
            r#""#
        };
    }
    macro_rules! WASM_BUILD_TEMPLATE_BUILD_RS {
        () => {
            r#"fn main() {
    risc0_build::embed_methods();
}"#
        };
    }
    macro_rules! WASM_BUILD_TEMPLATE_GUEST_CARGO_TOML {
        () => {
            r#"[package]
name = "wasm-guest"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
risc0-zkvm = { version = "0.17.0", default-features = false, features = ["std"] }
wasmi = "0.31.0""#
        };
    }
    macro_rules! WASM_BUILD_TEMPLATE_GUEST_MAIN_RS {
      () => {
          r#"#![no_main]
  
use risc0_zkvm::guest::env;
use wasmi::{{Engine, Linker, Module, Store}};

risc0_zkvm::guest::entry!(main);

const WASM_NAME: &str = "{}";
type WasmParams = {};
type WasmResult = {};

pub fn main() {{
    // Load environment variables
    let wasm_name = WASM_NAME;
    let wasm_file: Vec<u8> = env::read();
    let wasm_params: WasmParams = env::read();

    let engine = Engine::default();
    let module = Module::new(&engine, &mut &wasm_file[..]).expect("Failed to create module");
    type HostState = i32;

    let linker = <Linker<HostState>>::new(&engine);
    let mut store = Store::new(&engine, 0);
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instansitate.")
        .start(&mut store)
        .expect("Failed to start.");

    let wasm_fn = instance
        .get_typed_func::<WasmParams, WasmResult>(&store, WASM_NAME)
        .expect("Failed to get typed_func.");

    let res = wasm_fn
        .call(&mut store, wasm_params)
        .expect("Failed to call verify state");

    env::log(&format!(
        "Compile WASM {{}} - {{:?}}- {{}}",
        wasm_name, wasm_params, res
    ));

    env::commit(&res);
}}"#
      };
  }

    pub(crate) use WASM_BUILD_TEMPLATE_BUILD_RS;
    pub(crate) use WASM_BUILD_TEMPLATE_CARGO_TOML;
    pub(crate) use WASM_BUILD_TEMPLATE_GUEST_CARGO_TOML;
    pub(crate) use WASM_BUILD_TEMPLATE_GUEST_MAIN_RS;
    pub(crate) use WASM_BUILD_TEMPLATE_LIB_RS;
}
