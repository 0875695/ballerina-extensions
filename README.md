# Ballerina Extension & Tree-sitter Parser for Zed IDE

This repository provides support for the [Ballerina](https://ballerina.io/) programming language (Swan Lake) in the [Zed](https://zed.dev/) editor. It integrates a Tree-sitter parser for syntax highlighting, official Ballerina Language Server (LSP) integration, and a DAP debugger adapter proxy.

## Features

- **Syntax Highlighting**: Comprehensive Tree-sitter grammar supporting Ballerina Swan Lake syntax, including modern Ballerina 2024R1 constructs.
- **Language Server (LSP)**: Automatic detection and integration with the Ballerina Language Server (`bal start-language-server`).
- **Debugger (DAP)**: Debug adapter proxy supporting breakpoints, stepping, call stacks, and variable evaluation.

## Project Structure

- `grammar.js` — Tree-sitter grammar definition for Ballerina.
- `src/lib.rs` — Rust extension code implementing `zed_extension_api` to launch Ballerina LSP and the DAP proxy.
- `extension.toml` — Zed extension manifest (extension ID, version, language server and debugger definitions).
- `languages/ballerina/` — Zed language metadata, brackets, indentation, and highlight queries (`config.toml`, `highlights.scm`, `injections.scm`).
- `queries/highlights.scm` — Tree-sitter query rules for code highlighting.
- `test/corpus/` — Tree-sitter test corpus suite.
- `package.json` — NPM configuration for generating and testing the Tree-sitter grammar.
- `Cargo.toml` — Rust crate manifest for building the extension Wasm module.

## Ballerina 2024R1 Specification Support

The grammar and extension support the **Ballerina 2024R1** specification (Swan Lake Update 9+), including:

### 1. Alternate Receive Action
Concurrent wait from multiple workers:
```ballerina
string|error result = <- w1 | w2;
```
Receives the result from whichever worker finishes first.

### 2. Natural Expressions (Experimental)
Natural language template blocks with interpolated expressions for Generative AI / LLM workflows:
```ballerina
Joke joke = natural { 
    Tell a joke about ${subject}. 
};
```
The parser recognizes the `natural` keyword, braces `{}`, and `${expression}` substitutions within the block.

### 3. Extended Destructuring and Binding Patterns
- Tuple and record destructuring binding patterns.
- Error binding patterns with named error details.
- Rest binding patterns (`...rest`).

### 4. Arrow Functions & Anonymous Functions
Full support for modern closures and lambda syntax:
```ballerina
var add = (int a, int b) => a + b;
```

## Requirements

- **Ballerina Swan Lake** (Update 9 / 2201.9.0 or later recommended).
  Ensure `bal` is in your `PATH`, or specify the path in Zed settings.
- **Node.js** (optional, runtime for the DAP debug adapter proxy).

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

## Installation & Development

### Tree-sitter Grammar Development

Prerequisites: [Node.js](https://nodejs.org/) and `tree-sitter-cli`:

1. Install dependencies:
   ```bash
   npm install
   ```
2. Generate the parser:
   ```bash
   npx tree-sitter generate
   ```
3. Run grammar tests:
   ```bash
   npm test
   ```

### Building the Zed Extension

Prerequisites: [Rust](https://rustup.rs/) with the `wasm32-wasip1` target:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

To test locally in Zed, open the Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`), select **zed: install dev extension**, and choose this directory.

## License

This project is licensed under the [MIT License](LICENSE.md).
