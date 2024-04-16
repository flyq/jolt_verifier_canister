# jolt_verifier_canister

For implementation details, refer [here](https://hackmd.io/@liquan/S1dybGcl0).


The canister is already running on the chain: https://p6xvw-7iaaa-aaaap-aaana-cai.raw.ic0.app/

## Requirement

[ic-wasm](https://github.com/dfinity/ic-wasm)
```sh
cargo install ic-wasm -f
```

## Build

```sh
git clone https://github.com/flyq/jolt_verifier_canister

cd jolt_verifier_canister

cargo run --features "export-api" > build/jolt_verifier_canister.did

cargo build --target wasm32-unknown-unknown --release --features "export-api"

ic-wasm target/wasm32-unknown-unknown/release/jolt_verifier_canister.wasm -o build/jolt_verifier_canister.wasm shrink
```

## Deploy
teminal 1:
```sh
dfx start --clean
```

teminal 2:
```sh
dfx canister create --no-wallet jolt_verifier_canister

dfx build jolt_verifier_canister

dfx canister install jolt_verifier_canister --argument "record { owner=principal \"$(dfx identity get-principal)\";}"

# dfx canister install jolt_verifier_canister --argument "record { owner=principal \"yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae\"; ecdsa_env=variant {TestKeyLocalDevelopment}}" --upgrade-unchanged -m=upgrade 

dfx canister call jolt_verifier_canister get_owner
```

## Test

genenrate proof: 

terminal 2:
```sh
cd ..

git clone https://github.com/flyq/jolt

cd jolt

cargo run --release -p fibonacci

cargo run --release -p sha2-chain

cargo run --release -p sha2-ex

cargo run --release -p sha3-chain

cargo run --release -p sha3-ex
```

and we can get the proof files: fib10 and so on.
copy the file to `./data/fib`, `./data/sha2`, `data/sha2_chain`, `data/sha3`, `data/sha3_chain`.

also we get the risc-v compiled result under `ls /tmp/jolt-guest-*`.


generate the data according proof:

```sh
cargo run --release -p helper generate_preprocess guest fib
cargo run --release -p helper generate_preprocess sha2-chain-guest sha2_chain
cargo run --release -p helper generate_preprocess sha2-guest sha2
cargo run --release -p helper generate_preprocess sha3-chain-guest sha3_chain
cargo run --release -p helper generate_preprocess sha3-guest sha3

cargo run --release -p helper check_split

```

upload data
```sh
cargo run --release -p helper upload_preprocess

dfx canister call jolt_verifier_canister get_buffer '(24:nat32)'

# fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
dfx canister call jolt_verifier_canister preprocessing '(24:nat32, 0:nat32)'

dfx canister call jolt_verifier_canister get_buffer '(24:nat32)'

# modify helper/src/main.rs's upload_preprocess to `let name = format!("data/sha2/p{}.bin", i);`
cargo run --release -p helper upload_preprocess

dfx canister call jolt_verifier_canister get_buffer '(24:nat32)'

# sha2_ex(2)
dfx canister call jolt_verifier_canister preprocessing '(24:nat32, 2:nat32)'

dfx canister call jolt_verifier_canister get_buffer '(24:nat32)'

# modify helper/src/main.rs's upload_preprocess to `let name = format!("data/sha2_chain/p{}.bin", i);`
cargo run --release -p helper upload_preprocess

# sha2_chain(1)
dfx canister call jolt_verifier_canister preprocessing '(24:nat32, 1:nat32)'

# modify helper/src/main.rs's upload_preprocess to `let name = format!("data/sha3_chain/p{}.bin", i);`
cargo run --release -p helper upload_preprocess

# sha3_chain(3)
dfx canister call jolt_verifier_canister preprocessing '(24:nat32, 3:nat32)'

# modify helper/src/main.rs's upload_preprocess to `let name = format!("data/sha3/p{}.bin", i);`
cargo run --release -p helper upload_preprocess

# sha3(4)
dfx canister call jolt_verifier_canister preprocessing '(24:nat32, 4:nat32)'



# let name = format!("data/fib/fib10.bin");
cargo run --release -p helper upload_proof

# fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
dfx canister call jolt_verifier_canister update_proof '(0:nat32, 0:nat32)'
# (variant { Ok = 0 : nat32 })

# let name = format!("data/fib/fib50.bin");
cargo run --release -p helper upload_proof

# fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
dfx canister call jolt_verifier_canister update_proof '(0:nat32, 0:nat32)'
# (variant { Ok = 1 : nat32 })

# let name = format!("data/sha2/sha2.bin");
cargo run --release -p helper upload_proof

# fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
dfx canister call jolt_verifier_canister update_proof '(0:nat32, 2:nat32)'
# (variant { Ok = 0 : nat32 })

# let name = format!("data/sha3/sha3.bin");
cargo run --release -p helper upload_proof

# fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
dfx canister call jolt_verifier_canister update_proof '(0:nat32, 4:nat32)'
# (variant { Ok = 0 : nat32 })

# fib(10)
dfx canister call jolt_verifier_canister verify_jolt_proof '(0:nat32, 0:nat32)'
(variant { Ok = true })

# fib(50)
dfx canister call jolt_verifier_canister verify_jolt_proof '(0:nat32, 1:nat32)'

# sha2
dfx canister call jolt_verifier_canister verify_jolt_proof '(2:nat32, 0:nat32)'

Error: Failed update call.
Caused by: Failed update call.
  The replica returned a rejection error: reject code CanisterError, reject message Canister bnz7o-iuaaa-aaaaa-qaaaa-cai exceeded the instruction limit for single message execution., error code None

# sha3
dfx canister call jolt_verifier_canister verify_jolt_proof '(4:nat32, 0:nat32)'

Error: Failed update call.
Caused by: Failed update call.
  The replica returned a rejection error: reject code CanisterError, reject message Canister bnz7o-iuaaa-aaaaa-qaaaa-cai exceeded the instruction limit for single message execution., error code None
```

It can be seen that the verification of sha2 and sha3 does need to wait for [the optimization of Verifier](https://github.com/a16z/jolt/issues/216)