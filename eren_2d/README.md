
## WASM 빌드
```
cargo build --target wasm32-unknown-unknown --release
wasm-opt ./target/wasm32-unknown-unknown/release/eren_2d.wasm -O4 --flatten --merge-blocks --simplify-globals --rereloop -o ./target/wasm32-unknown-unknown/release/eren_2d_opt.wasm
wasm-bindgen --out-dir ./wasm --target web ./target/wasm32-unknown-unknown/release/eren_2d_opt.wasm
```
