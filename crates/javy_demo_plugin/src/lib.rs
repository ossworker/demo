use javy_plugin_api::{import_namespace, Config};
use std::slice;

import_namespace!("javy_quickjs_provider_v3");

/// Used by Wizer to preinitialize the module.
#[export_name = "initialize_runtime"]
pub extern "C" fn initialize_runtime() {
    let mut config = Config::default();
    config
        .event_loop(true)
        .text_encoding(true)
        .redirect_stdout_to_stderr(true)
        .javy_stream_io(true)
        .simd_json_builtins(true)
        .javy_json(true);

    javy_plugin_api::initialize_runtime(config, |runtime| {
        runtime
            .context()
            .with(|ctx| ctx.globals().set("plugin", true))
            .unwrap();
        runtime
    })
    .unwrap();
}

#[export_name = "eval_bytecode"]
pub unsafe extern "C" fn eval_bytecode(bytecode_ptr: *const u8, bytecode_len: usize) {
    let bytecode = slice::from_raw_parts(bytecode_ptr, bytecode_len);
    javy_plugin_api::run_bytecode(bytecode, None);
}
