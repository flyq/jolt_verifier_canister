use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use ark_bn254::{Fr, G1Projective};
use ark_serialize::CanonicalSerialize;
use ark_serialize::Read;
use candid::Encode;
use ic_agent::{export::Principal, Agent};
use jolt_core::host::Program;
use jolt_core::jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt, JoltPreprocessing};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "generate_preprocess" => preprocess(),
        "call_preprocess" => call_preprocess().await,
        "call_verify" => call_verify().await,
        _ => panic!("Invalid command"),
    }
}

fn preprocess() {
    let mut program = Program::new("guest");
    program.set_func("fib");
    program.elf = Some(
        PathBuf::from_str(
            "/tmp/jolt-guest-target-fibonacci-guest-fib/riscv32i-unknown-none-elf/release/guest",
        )
        .unwrap(),
    );
    let (bytecode, memory_init) = program.decode();

    let preprocessing: JoltPreprocessing<Fr, G1Projective> =
        RV32IJoltVM::preprocess(bytecode, memory_init, 1 << 10, 1 << 10, 1 << 14);

    let file = File::create("preprocess.bin").unwrap();
    preprocessing.serialize_compressed(file).unwrap();
}

async fn call_preprocess() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let mut file = File::open("preprocess.bin").unwrap();
    let mut preprocess = Vec::new();
    file.read_to_end(&mut preprocess).unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    let res = agent
        .update(&canister_id, "preprocessing")
        .with_arg(Encode!(&preprocess).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    println!("{:?}", res);
}

async fn call_verify() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let mut file = File::open("proof.bin").unwrap();
    let mut proof = Vec::new();
    file.read_to_end(&mut proof).unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    let res = agent
        .query(&canister_id, "verify_jolt_proof")
        .with_arg(Encode!(&proof).unwrap())
        .call()
        .await
        .unwrap();

    println!("{:?}", res);
}
