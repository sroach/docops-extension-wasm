# DocOps Extension WASM


## Overview

This project is an extension for DocOps, a tool for generating documentation from code. It provides a WebAssembly (WASM) implementation of the extension, allowing it to be used in a web browser.

## Usage

To use this extension, you can include the generated WASM file in your DocOps project and configure it to use the extension. For more information on how to use the extension, please refer to the [DocOps documentation](https://docops.io).

## Building

Rust is required to build the extension from source. You can install Rust by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

If you want to build the extension from source, you can run the following command:

```shell
wasm-pack build --target web --release
```

### Launch 

```shell
python3 -m http.server 8000
```
open http://localhost:8000 in your browser

## Test

To test the extension, you can run the following command:

```shell
cargo test
```

### Demo page

https://sroach.github.io/docops-extension-wasm/

#### Building demo

```shell
wasm-pack build --target web --out-dir docs/pkg
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## Signing

The svg can be signed using one of the following approaches: 

control is privkey

You can generate a compatible key using any of the following methods:

### Using OpenSSL
   This is the fastest method if you have OpenSSL installed (standard on macOS and Linux):
```shell
openssl rand -hex 32
```

Output: A 64-character hex string like 0001020304...

### Using Python
   If you have Python installed, you can generate a random hex string with a one-liner:

```shell
python3 -c "import os; print(os.urandom(32).hex())"
````

### Using Node.js
```shell
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"
```

## Target Node

```shell
wasm-pack build --target nodejs --release --out-dir pkg-node
```