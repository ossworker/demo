# javy plugin

## build plugin

javy emit-plugin -o plugin.wasm

```shell
cargo build -r -p javy_demo_plugin --target wasm32-wasip1
javy init-plugin target/wasm32-wasip1/release/javy_demo_plugin.wasm -o crates/javy_demo_plugin/javy_demo_plugin_wizer.wasm
```

## test

### wasmtime

```shell
cd crates/javy_demo_plugin
echo 'console.log("hello world!",plugin);' > my_code.js
javy build -C dynamic -C plugin=javy_demo_plugin_wizer.wasm -o my_code.wasm my_code.js
wasmtime run --preload javy_quickjs_provider_v3=javy_demo_plugin_wizer.wasm my_code.wasm
```

out>
hello world! true

### nodejs

```shell
javy build -C dynamic -C plugin=javy_demo_plugin_wizer.wasm -o embedded.wasm embedded.js
node --no-warnings=ExperimentalWarning host.mjs
```

out>
Success! {
"n": 101,
"plugin": true
}

embedded.js

```javascript
// Read input from stdin
const input = readInput();
// Call the function with the input
const result = foo(input);
// Write the result to stdout
writeOutput(result);

// The main function.
function foo(input) {
  if (input && typeof input === "object" && typeof input.n === "number") {
    return { n: input.n + 1, plugin };
  }
  return { n: 0 };
}

// Read input from stdin
function readInput() {
  const chunkSize = 1024;
  const inputChunks = [];
  let totalBytes = 0;

  // Read all the available bytes
  while (1) {
    const buffer = new Uint8Array(chunkSize);
    // Stdin file descriptor
    const fd = 0;
    const bytesRead = Javy.IO.readSync(fd, buffer);

    totalBytes += bytesRead;
    if (bytesRead === 0) {
      break;
    }
    inputChunks.push(buffer.subarray(0, bytesRead));
  }

  // Assemble input into a single Uint8Array
  const { finalBuffer } = inputChunks.reduce(
    (context, chunk) => {
      context.finalBuffer.set(chunk, context.bufferOffset);
      context.bufferOffset += chunk.length;
      return context;
    },
    { bufferOffset: 0, finalBuffer: new Uint8Array(totalBytes) }
  );

  const maybeJson = new TextDecoder().decode(finalBuffer);
  try {
    return JSON.parse(maybeJson);
  } catch {
    return;
  }
}

// Write output to stdout
function writeOutput(output) {
  const encodedOutput = new TextEncoder().encode(JSON.stringify(output));
  const buffer = new Uint8Array(encodedOutput);
  // Stdout file descriptor
  const fd = 1;
  Javy.IO.writeSync(fd, buffer);
}
```

host.mjs

```javascript
import { readFile, writeFile, open } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { WASI } from "wasi";

try {
  const [embeddedModule, pluginModule] = await Promise.all([
    compileModule("./embedded.wasm"),
    compileModule("./javy_demo_plugin_wizer.wasm"),
  ]);
  const result = await runJavy(pluginModule, embeddedModule, { n: 100 });
  console.log("Success!", JSON.stringify(result, null, 2));
} catch (e) {
  console.log(e);
}

async function compileModule(wasmPath) {
  const bytes = await readFile(new URL(wasmPath, import.meta.url));
  return WebAssembly.compile(bytes);
}

async function runJavy(pluginModule, embeddedModule, input) {
  const uniqueId = crypto.randomUUID();

  // Use stdin/stdout/stderr to communicate with Wasm instance
  // See https://k33g.hashnode.dev/wasi-communication-between-nodejs-and-wasm-modules-another-way-with-stdin-and-stdout
  const workDir = tmpdir();
  const stdinFilePath = join(workDir, `stdin.wasm.${uniqueId}.txt`);
  const stdoutFilePath = join(workDir, `stdout.wasm.${uniqueId}.txt`);
  const stderrFilePath = join(workDir, `stderr.wasm.${uniqueId}.txt`);

  // 👋 send data to the Wasm instance
  await writeFile(stdinFilePath, JSON.stringify(input), { encoding: "utf8" });

  const [stdinFile, stdoutFile, stderrFile] = await Promise.all([
    open(stdinFilePath, "r"),
    open(stdoutFilePath, "a"),
    open(stderrFilePath, "a"),
  ]);

  try {
    const wasi = new WASI({
      version: "preview1",
      args: [],
      env: {},
      stdin: stdinFile.fd,
      stdout: stdoutFile.fd,
      stderr: stderrFile.fd,
      returnOnExit: true,
    });

    const pluginInstance = await WebAssembly.instantiate(
      pluginModule,
      wasi.getImportObject()
    );
    const instance = await WebAssembly.instantiate(embeddedModule, {
      javy_quickjs_provider_v3: pluginInstance.exports,
    });

    // Javy plugin is a WASI reactor see https://github.com/WebAssembly/WASI/blob/main/legacy/application-abi.md?plain=1
    wasi.initialize(pluginInstance);
    instance.exports._start();

    const [out, err] = await Promise.all([
      readOutput(stdoutFilePath),
      readOutput(stderrFilePath),
    ]);

    if (err) {
      throw new Error(err);
    }

    return out;
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      const errorMessage = await readOutput(stderrFilePath);
      if (errorMessage) {
        throw new Error(errorMessage);
      }
    }
    throw e;
  } finally {
    await Promise.all([
      stdinFile.close(),
      stdoutFile.close(),
      stderrFile.close(),
    ]);
  }
}

async function readOutput(filePath) {
  const str = (await readFile(filePath, "utf8")).trim();
  try {
    return JSON.parse(str);
  } catch {
    return str;
  }
}
```

echo 'console.log("hello world!");' > my_code.js
javy emit-plugin -o plugin.wasm
javy build -C dynamic -C plugin=plugin.wasm -o my_code.wasm my_code.js
wasmtime run --preload javy_quickjs_provider_v3=plugin.wasm my_code.wasm
hello world!

ci
cargo build --package=javy-test-plugin --release --target=wasm32-wasip1
CARGO_PROFILE_RELEASE_LTO=off cargo build --package=javy-cli --release
target/release/javy init-plugin target/wasm32-wasip1/release/test_plugin.wasm -o crates/runner/test_plugin.wasm
