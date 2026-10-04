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

### Launch Demo Page Locally

```shell
python3 -m http.server 8000
```
open http://localhost:8000 in your browser

## HTTP Server (On-Demand SVG API)

The project includes a high-performance native Rust microservice (`docops-server`) built with [Axum](https://github.com/tokio-rs/axum) for generating SVGs on demand via HTTP / REST URLs.

### Running the Server

To start the server locally:

```shell
cargo run --features server --bin docops-server
```

For production / optimized builds:

```shell
cargo run --release --features server --bin docops-server
```

### Configuration

The server can be configured using environment variables:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `PORT` | `3000` | Port to bind the server |
| `HOST` | `0.0.0.0` | Host IP address |
| `RUST_LOG` | `docops_server=info,tower_http=info` | Tracing / logging filter level |

Example:
```shell
PORT=8080 HOST=127.0.0.1 cargo run --release --features server --bin docops-server
```

### API Endpoints

- `GET /` — Interactive overview & documentation page.
- `GET /health` — Service health check (returns `200 OK`).
- `GET /svg?type=<type>&data=<payload>` — Render SVG from query parameters.
- `GET /svg/:type/:payload` — Render SVG using path parameters.
- `GET /svg/:payload` — Render SVG from full encoded `[docops,...]` block.
- `POST /svg` — Render visual from raw DSL string (`text/plain`) or JSON payload (`application/json`).
- `POST /svg/:type` — Render visual body for a specific type.

### Payload Encoding

Payloads can be encoded using:
1. **URL-Safe Base64** (or standard Base64) with **Deflate compression** (recommended for compact URLs).
2. **URL-Safe Base64** with **Zlib** or **Gzip** compression.
3. **Plain Base64** (uncompressed text).

#### Client-side JavaScript / Node.js Encoding Example (Native `CompressionStream`)

Using the native Web Streams API (`CompressionStream`), built into all modern browsers and Node.js 18+ (zero external dependencies required):

```javascript
// Compress using native CompressionStream and encode to URL-safe Base64
async function encodeDocOps(text, format = "deflate-raw") {
  const stream = new Blob([text]).stream().pipeThrough(new CompressionStream(format));
  const compressed = await new Response(stream).arrayBuffer();
  const bytes = new Uint8Array(compressed);

  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }

  // Convert standard Base64 to URL-safe Base64
  return btoa(binary)
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

// Example usage:
const pieBody = `----
theme=premium
title=Website Traffic
---
Product A | 30
Product B | 70
----`;

const encoded = await encodeDocOps(pieBody);
const url = `http://localhost:3000/svg?type=pie&data=${encoded}`;
```

#### Embedding in Markdown

```markdown
![Website Traffic](http://localhost:3000/svg?type=pie&data=eNqLVjDXM9Qz0TMw1DMwM...)
```

### Testing the Server

To run the server unit and integration tests:

```shell
cargo test --features server --bin docops-server
```

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