// #[allow(warnings)]
// mod bindings;

// use bindings::Guest;

// #[warn(dead_code)]
// struct Component;

// impl Guest for Component {
//     fn greeting(name: String) -> String {
//         println!("input->{:#?}", name);
//         "Hello World".to_string()
//     }
// }

// bindings::export!(Component with_types_in bindings);

pub mod bindings {
    wit_bindgen::generate!({
        path: "./wit",
        world: "workoss:new-world/new-world",
        pub_export_macro: true,
        export_macro_name: "export",
        generate_all,
        // with: {
        //     "wasi:cli/environment@0.2.2": generate,
        //     "wasi:cli/exit@0.2.2": generate,
        //     "wasi:cli/stdin@0.2.2": generate,
        //     "wasi:cli/stdout@0.2.2": generate,
        //     "wasi:cli/stderr@0.2.2": generate,
        //     "wasi:cli/terminal-input@0.2.2": generate,
        //     "wasi:cli/terminal-output@0.2.2": generate,
        //     "wasi:cli/terminal-stdin@0.2.2": generate,
        //     "wasi:cli/terminal-stdout@0.2.2": generate,
        //     "wasi:cli/terminal-stderr@0.2.2": generate,
        //     "wasi:cli/run@0.2.2": generate,
        //     "wasi:io/error@0.2.2": generate,
        //     "wasi:io/poll@0.2.2": generate,
        //     "wasi:io/streams@0.2.2": generate,
        //     "wasi:clocks/monotonic-clock@0.2.2": generate,
        //     "wasi:clocks/wall-clock@0.2.2": generate,
        //     "wasi:filesystem/types@0.2.2": generate,
        //     "wasi:filesystem/preopens@0.2.2": generate,
        //     "wasi:sockets/network@0.2.2": generate,
        //     "wasi:sockets/instance-network@0.2.2": generate,
        //     "wasi:sockets/udp@0.2.2": generate,
        //     "wasi:sockets/udp-create-socket@0.2.2": generate,
        //     "wasi:sockets/tcp@0.2.2": generate,
        //     "wasi:sockets/tcp-create-socket@0.2.2": generate,
        //     "wasi:sockets/ip-name-lookup@0.2.2": generate,
        //     "wasi:random/random@0.2.2": generate,
        //     "wasi:random/insecure@0.2.2": generate,
        //     "wasi:random/insecure-seed@0.2.2": generate,
        // }
    });
}

pub use crate::bindings::{export, Guest as Plugin};
