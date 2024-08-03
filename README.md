# Vara Wordle Game Contract

### 🏗️ Building

```sh
cargo build --release -p session_game
cargo build --release -p wordle_game
```

### ✅ Testing

Run all tests, except `gclient` ones:
```sh
cargo t --workspace -- --skip gclient
```

Run all tests:
```sh
# Download the node binary.
cargo xtask node
cargo t --workspace