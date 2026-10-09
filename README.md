# Ballerina Extension for Zed

This extension provides support for the [Ballerina](https://ballerina.io/) programming language (Swan Lake) in the [Zed](https://zed.dev/) editor. It includes Tree-sitter grammar for syntax highlighting, integration with the official Ballerina Language Server (LSP), and a DAP debugger adapter.

## Features

- **Syntax Highlighting**: Comprehensive Tree-sitter grammar supporting Ballerina Swan Lake syntax, including modern Ballerina 2024R1 constructs.
- **Language Server (LSP)**: Automatic detection and integration with the Ballerina Language Server (`bal start-language-server`).
- **Debugger (DAP)**: Debug adapter proxy supporting breakpoints, stepping, call stacks, and variable evaluation.

## Requirements

- **Ballerina Swan Lake** (Update 9 / 2201.9.0 or later recommended).
  Ensure `bal` is available in your `PATH`, or configure the path in Zed settings.
- **Node.js** (optional, used as runtime for the debug adapter proxy).

## Configuration

You can configure the extension in your Zed `settings.json`:

```json
{
  "lsp": {
    "ballerina-language-server": {
      "binary": {
        "path": "/path/to/bal"
      }
    }
  }
}
```

## Development

### Building and Testing the Tree-sitter Grammar

Prerequisites: [Node.js](https://nodejs.org/) and `tree-sitter-cli`.

1. Install dependencies:
   ```bash
   npm install
   ```
2. Generate the parser:
   ```bash
   npx tree-sitter generate
   ```
3. Run tests:
   ```bash
   npm test
   ```

### Building the Zed Extension

Prerequisites: [Rust](https://rustup.rs/) with the `wasm32-wasip1` target:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

To test locally in Zed, open the Command Palette (`cmd-shift-p` / `ctrl-shift-p`) and select **zed: install dev extension**, then choose this directory.

## License

This project is licensed under the [MIT License](LICENSE.md).
