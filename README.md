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