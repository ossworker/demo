#![deny(warnings)]

use anyhow::Ok;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store,
};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{bindings::ProxyPre, WasiHttpCtx, WasiHttpView};

struct MyClientState {
    table: ResourceTable,
    wasi: WasiCtx,
    wasi_http: WasiHttpCtx,
}

impl WasiView for MyClientState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for MyClientState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.wasi_http
    }
}

struct MyServer {
    pre: ProxyPre<MyClientState>,
}

fn main() -> anyhow::Result<()> {
    let mut config = Config::new();
    config.async_support(true);
    config.wasm_component_model(true);
    config.allocation_strategy(InstanceAllocationStrategy::Pooling(
        PoolingAllocationConfig::default(),
    ));

    let engine = Engine::new(&config)?;

    let path = "/Users/workoss/IDE/rustProjects/demo/target/wasm32-wasip2/release/http_guest.wasm";

    let component = Component::from_file(&engine, path)?;

    let mut linker = Linker::new(&engine);

    wasmtime_wasi::add_to_linker_async(&mut linker)?;

    wasmtime_wasi_http::add_to_linker_async(&mut linker)?;

    let pre = ProxyPre::new(linker.instantiate_pre(&component)?)?;

    let my_client_state = MyClientState {
        table: ResourceTable::new(),
        wasi: WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(".", ".", DirPerms::READ, FilePerms::READ)?
            .build(),
        wasi_http: WasiHttpCtx::new(),
    };

    let store = Store::new(&engine, my_client_state);

    Ok(())
}
