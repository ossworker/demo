use std::env;

use wasmtime::{
    component::{bindgen, Component, Linker, ResourceTable},
    Config, Engine, Result, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::new();
    config.async_support(true);
    config.wasm_component_model(true);
    config.debug_info(true);
    let engine = Engine::new(&config)?;

    bindgen!({
        world: "new-world",
        path: "../plugin_api/wit",
        // tracing: true,
        async: {
            only_imports: ["nonexistent"],
        },
        trappable_imports: true,
        // with: { "wasi": wasmtime_wasi::bindings::exports::wasi },
        // ownership: Borrowing {
        //     duplicate_if_necessary: false
        // }
    });

    let mut linker = Linker::new(&engine);

    // Add all the WASI extensions to the linker
    wasmtime_wasi::add_to_linker_async(&mut linker)?;

    //NewWorld::add_to_linker(&mut linker, |state: &mut MyState| state)?;

    // ... configure `builder` more to add env vars, args, etc ...
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdio();
    let mut store = Store::new(
        &engine,
        MyState {
            ctx: builder.build(),
            table: ResourceTable::new(),
        },
    );

    let current_dir = env::current_dir()?;

    let component = Component::from_file(
        &engine,
        current_dir
            .join("./target/wasm32-wasip1/release/custom_plugin.wasm")
            .as_path(),
    )?;
    // let component = Component::from_file(&engine, "../wasm32-wasip1/release/custom_plugin.wasm")?;
    let new_world = NewWorld::instantiate_async(&mut store, &component, &linker).await?;
    let greeting = new_world.call_greeting(&mut store, "Ben").await?;

    println!("{greeting}");

    Ok(())
}

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
