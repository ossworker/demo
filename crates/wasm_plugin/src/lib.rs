use wasm_plugin_api::{get_settings, register_plugin, Greeter, Message};

struct DemoGreeter;

impl Greeter for DemoGreeter {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn greet(&self, msg: Message) -> Result<String, String> {
        println!("Oh! {}, my name is {}", msg.content, msg.name);
        let settings = get_settings(Some("key"));
        let out = match settings {
            Ok(s) => s,
            Err(e) => e,
        };
        println!("settings: {:#?}", out);
        Ok("greet hello world".to_string())
    }
}

register_plugin!(DemoGreeter);
