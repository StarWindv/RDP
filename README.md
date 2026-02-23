# rs-dash-pro

A POSIX-compatible shell implementation in Rust with modern architecture (Lexer→Parser→IRGenerator→Optimizer→Executor).

## Architecture

```
Lexer → Parser → IRGenerator → Optimizer → Executor
```

### Components

1. **Lexer**: Tokenizes shell script into tokens
2. **Parser**: Parses tokens into AST (Abstract Syntax Tree)
3. **IRGenerator**: Converts AST to IR (Intermediate Representation)
4. **Optimizer**: Optimizes IR (currently empty implementation)
5. **Executor**: Executes optimized IR

## Features

### ✅ Implemented
- Basic tokenization
- Simple command parsing
- Environment variable support
- Command execution

### 🔄 In Progress
- Full POSIX grammar support
- Pipeline and redirection
- Control structures
- Functions

## Building

```bash
cargo build
cargo build --release
```

## Running

```bash
# Interactive mode
cargo run

# Execute command
cargo run -- -c "echo hello"

# Execute script
cargo run -- script.sh
```

## Testing

```bash
cargo test
```

## License

MIT