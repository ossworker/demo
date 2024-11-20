use std::{path::Path, time::Instant};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{async_trait, ResourceTable, WasiCtx, WasiView};

wasmtime::component::bindgen!({
    path: "../wasm_plugin_api/wit",
    world: "extension",
    async: true,
    trappable_imports: true,
});

struct WasmState {
    pub table: ResourceTable,
    ctx: wasmtime_wasi::WasiCtx,
}

impl WasiView for WasmState {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.ctx
    }
}

#[async_trait]
impl ExtensionImports for WasmState {
    async fn get_settings(
        &mut self,
        key: Option<String>,
    ) -> wasmtime::Result<Result<String, String>> {
        println!("import key:{:#?}", key.unwrap());
        Ok(Ok("get_settings_value".to_string()))
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let now = Instant::now();
    let mut config = Config::new();
    config.async_support(true);
    config.wasm_component_model(true);
    config.debug_info(true);
    let engine = Engine::new(&config)?;

    let current_dir = env!("CARGO_MANIFEST_DIR");
    let component = Component::from_file(
        &engine,
        Path::new(current_dir)
            .join("../../target/wasm32-wasip2/release/wasm_plugin.wasm")
            .as_path(),
    )?;

    let mut linker = Linker::<WasmState>::new(&engine);

    Extension::add_to_linker(&mut linker, |s: &mut WasmState| s)?;

    wasmtime_wasi::add_to_linker_async(&mut linker)?;

    let wasm_state = WasmState {
        table: ResourceTable::new(),
        ctx: WasiCtx::builder().inherit_stdio().build(),
    };

    let mut store = Store::new(&engine, wasm_state);

    let extension = Extension::instantiate_async(&mut store, &component, &linker).await?;

    let msg = Message {
        content: "hello world from wasm!".to_string(),
        name: "张三".to_string(),
    };

    extension.call_init_plugin(&mut store).await?;

    let greet_result = extension.call_greet(&mut store, &msg).await?;

    println!("greet_result:{:#?}", greet_result.unwrap());

    let cost_millis = Instant::now().duration_since(now).as_millis();
    println!("cost:{cost_millis}ms");

    Ok(())
}
