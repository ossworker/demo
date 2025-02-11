# wasi-http

安装 wasmtime

```shell
curl https://wasmtime.dev/install.sh -sSf | bash
```

安装 cargo-component

```shell
cargo install cargo-component
```

```shell
cargo component build -r
wasmtime serve target/wasm32-wasip1/release/wasi_http_demo.wasm
```
