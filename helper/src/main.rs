use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use ark_bn254::{Fr, G1Projective};
use ark_serialize::CanonicalSerialize;
use ark_serialize::Read;
use ark_serialize::Write;
use candid::{Decode, Encode};
use ic_agent::{export::Principal, Agent};
use jolt_core::host::Program;
use jolt_core::jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt, JoltPreprocessing};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "generate_preprocess" => generate_preprocess(),
        "check_split" => check_split(),
        "upload_preprocess" => upload_preprocess().await,
        "call_preprocess" => call_preprocess().await,
        "update_proof" => update_proof().await,
        _ => panic!("Invalid command"),
    }
}

fn generate_preprocess() {
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

    let mut buffer: Vec<u8> = Vec::new();
    preprocessing.serialize_compressed(&mut buffer).unwrap();
    println!("buffer size: {}", buffer.len());

    for i in 0..25 {
        let name = format!("data/p{}.bin", i);
        let mut file = File::create(name).unwrap();

        if i < 24 {
            let temp = &buffer[i * 2_000_000..(i + 1) * 2_000_000];
            // write temp to file
            file.write_all(temp).unwrap();
        } else {
            let temp = &buffer[i * 2_000_000..];
            // write temp to file
            file.write_all(temp).unwrap();
        }
    }
    let file = File::create("data/preprocess.bin").unwrap();
    preprocessing.serialize_compressed(file).unwrap();
}

fn check_split() {
    let mut file = File::open("data/preprocess.bin").unwrap();
    let mut preprocess = Vec::new();
    file.read_to_end(&mut preprocess).unwrap();

    let mut buffer: Vec<u8> = Vec::new();

    for i in 0..25 {
        let name = format!("data/p{}.bin", i);
        let mut file = File::open(name).unwrap();
        let mut temp = Vec::new();
        file.read_to_end(&mut temp).unwrap();
        buffer.extend(temp);
    }

    assert_eq!(buffer, preprocess);

    println!("Split and merge successfully");
}

async fn upload_preprocess() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    for i in 0..25u32 {
        let name = format!("data/p{}.bin", i);
        let mut file = File::open(name).unwrap();
        let mut temp = Vec::new();
        file.read_to_end(&mut temp).unwrap();

        println!("{}th, part preprocess size: {}", i, temp.len());

        let res = agent
            .update(&canister_id, "upload_preprocessing_buffer")
            .with_arg(Encode!(&i, &temp).unwrap())
            .call_and_wait()
            .await
            .unwrap();

        println!("{:?}", Decode!(&res));
    }
}

async fn call_preprocess() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    let res = agent
        .update(&canister_id, "preprocessing")
        .with_arg(Encode!().unwrap())
        .call_and_wait()
        .await
        .unwrap();

    println!("{:?}", Decode!(&res));
}

async fn update_proof() {
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
        .update(&canister_id, "update_proof")
        .with_arg(Encode!(&proof).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    println!("{:?}", Decode!(&res));
}
