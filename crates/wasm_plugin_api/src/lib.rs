use std::sync::{Mutex, MutexGuard};

pub trait Greeter: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;

    fn greet(&self, msg: Message) -> Result<String, String>;
}

wit_bindgen::generate!({
    path: "./wit",
    world: "extension",
    skip: ["init-plugin"],
});

struct Component;

export!(Component);

impl Guest for Component {
    #[doc = " greet"]
    fn greet(msg: Message) -> Result<String, String> {
        let instance = get_plugin();
        if let Ok(p) = instance {
            return p.as_ref().unwrap().greet(msg);
        }
        Err("get plugin error".to_string())
    }
}

static PLUGIN: Mutex<Option<Box<dyn Greeter>>> = Mutex::new(None);

#[macro_export]
macro_rules! register_plugin {
    ($extension_type:ty) => {
        #[export_name = "init-plugin"]
        pub extern "C" fn __init_extension() {
            let _ = wasm_plugin_api::load_plugin(Box::new(
                <$extension_type as wasm_plugin_api::Greeter>::new(),
            ));
        }
    };
}

pub fn load_plugin(greeter: Box<dyn Greeter>) -> anyhow::Result<()> {
    let mut binding = PLUGIN.lock();
    let plugin = binding.as_mut();
    if let Ok(p) = plugin {
        **p = Some(greeter);
    } else {
        anyhow::bail!("Fail to load the plugin");
    }

    Ok(())
}

fn get_plugin<'a>() -> anyhow::Result<MutexGuard<'a, Option<Box<dyn Greeter>>>> {
    let plugin = PLUGIN.lock();
    if let Ok(p) = plugin {
        Ok(p)
    } else {
        anyhow::bail!("Failed to use the plugin");
    }
}
