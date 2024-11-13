use plugin_api::bindings::export;
use plugin_api::Plugin;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn greeting(name: String) -> String {
        println!("STDIO WORKS!");

        // let config = Agent::config_builder()
        //     .timeout_global(Some(Duration::from_secs(5)))
        //     .tls_config(
        //         TlsConfig::builder()
        //             .provider(ureq::tls::TlsProvider::Rustls)
        //             .build(),
        //     )
        //     .build();

        // let agent = Agent::new_with_config(config);
        // let body: String = agent
        //     .get("http://httpbin.org/get")
        //     .call()
        //     .unwrap()
        //     .body_mut()
        //     .read_to_string()
        //     .unwrap();
        // println!("http:{body}");

        format!("Greetings {name}! I'm a WASI plugin!")
    }
}
export!(MyPlugin with_types_in plugin_api::bindings);
