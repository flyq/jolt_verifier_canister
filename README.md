# jolt_verifier_canister

## Requirement

[ic-wasm](https://github.com/dfinity/ic-wasm)
```sh
cargo install ic-wasm -f
```

## Build

```sh
cargo run --features "export-api" > build/jolt_verifier_canister.did

cargo build --target wasm32-unknown-unknown --release --features "export-api"

ic-wasm target/wasm32-unknown-unknown/release/jolt_verifier_canister.wasm -o build/jolt_verifier_canister.wasm shrink
```

## Deploy

```sh
dfx start --clean

dfx canister create --no-wallet jolt_verifier_canister

dfx build jolt_verifier_canister

dfx canister install jolt_verifier_canister --argument "record { owner=principal \"$(dfx identity get-principal)\";}"

# dfx canister install jolt_verifier_canister --argument "record { owner=principal \"yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae\"; ecdsa_env=variant {TestKeyLocalDevelopment}}" --upgrade-unchanged -m=upgrade 

dfx canister call jolt_verifier_canister get_owner

cargo run --release -p helper generate_preprocess

cargo run --release -p helper check_split
cargo run --release -p helper upload_preprocess

dfx canister call jolt_verifier_canister get_buffer '(24:nat32)'

# cargo run --release -p helper call_preprocess
dfx canister call jolt_verifier_canister preprocessing

cargo run --release -p helper update_proof

dfx canister call jolt_verifier_canister verify_jolt_proof
```