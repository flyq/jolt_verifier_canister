# jolt_verifier_canister

## background

A zero-knowledge proof is a way of proving the validity of a statement without revealing the statement itself. The ‘prover’ is the party trying to prove a claim, while the ‘verifier’ is responsible for validating the claim. It provides core technical support in use cases such as private transactions, verifiable computing (Off-chain scaling solutions, aka Layer2).

Thanks to the needs arising in the blockchain field, such as zkRollup, private transactions, etc., the theory and engineering in the ZKP field have developed rapidly in recent years. To use the power brought by ZKP, on-chain verifier are almost indispensable. For example, all Ethereum zkRollups has a Verifier implemented using Solidity running on Ethereum Layer1, and when withdrawing Tornado cash, you also need to verify the merkle proof you generated in the Tornado smart contract.

On April 9th, the A16z crypto team released the fastest zkVM for prover: Jolt(Just One Lookup Table), including [code](https://github.com/a16z/jolt), [examples](https://github.com/a16z/jolt/tree/main/examples), [documents](https://jolt.a16zcrypto.com/), [blogs](https://a16zcrypto.com/posts/tags/lasso-jolt), and papers([Lasso](https://eprint.iacr.org/2023/1216.pdf) and [Jolt](https://eprint.iacr.org/2023/1217.pdf)).

> Jolt is a zkVM (zero knowledge virtual machine) – a SNARK that lets the prover prove that it correctly ran a specified computer program, where the program is written in the assembly language of some simple CPU. zkVMs offer a fantastic developer experience: They make SNARKs usable by anyone who can write a computer program, eliminating the need for in-depth cryptographic knowledge. But usability comes at a steep price. Today’s zkVMs are remarkably complicated and offer terrible performance for the prover. Today, *proving* a computer program was run correctly is millions of times slower than simply *running* the program.
> 
> from [A new era in SNARK design: Releasing Jolt](https://a16zcrypto.com/posts/article/a-new-era-in-snark-design-releasing-jolt)

And the Jolt development team encourages the implementation of [on-chain Verifier](https://github.com/a16z/jolt/issues/209).

[IC(Internet Computer)](https://internetcomputer.org/) is the most powerful smart contract platform I have ever seen. One of its smart contracts can run a complete EVM, including json rpc, processing signatures, and EVM execution and output. Or being able to run a database, a sequencer of Bitcoin inscriptions, etc.

Putting zk's verifier on an IC might do some weird things.
1. IC is powerful enough and not so picky about the Verifier program. In fact, there is a balance between the prover and the verifier. Some SNARKs can obtain the smallest proof size and fast verification algorithm, but they are very unfriendly to the prover. Some SNARKs may have a larger proof size and a more complicated verification algorithm. But the performance of the proof program has been greatly improved. Due to the performance limitations of smart contracts, Ethereum can often only verify the former type of SNARK. But Sumcheck-based Jolt falls into the latter category.
2. IC's smart contract has the most complete support for Rust and can use std, so it can directly reuse the implementation of ZK Library. and reduce potential implementation errors.
3. IC's chain key ECDSA can directly call platforms such as Ethereum/BTC, and the verification results can also be used to notify Ethereum, etc.

For code implementation details, refer [here](https://hackmd.io/@liquan/S1dybGcl0).

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