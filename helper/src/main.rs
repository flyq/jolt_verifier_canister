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
        "generate_preprocess" => preprocess(),
        "upload_preprocess" => upload_preprocess().await,
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

    let mut buffer: Vec<u8> = Vec::new();
    preprocessing.serialize_compressed(&mut buffer).unwrap();
    println!("buffer size: {}", buffer.len());

    for i in 0..25 {
        let name = format!("preprocess/p{}.bin", i);
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
    let file = File::create("preprocess.bin").unwrap();
    preprocessing.serialize_compressed(file).unwrap();
}

async fn upload_preprocess() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    // let mut buffer = Vec::new();
    for i in 0..25u8 {
        let name = format!("preprocess/p{}.bin", i);
        let mut file = File::open(name).unwrap();
        let mut preprocess = Vec::new();
        file.read_to_end(&mut preprocess).unwrap();

        println!("preprocess size: {}", preprocess.len());

        // buffer.extend(preprocess);

        let res = agent
            .update(&canister_id, "upload_preprocessing_buffer")
            .with_arg(Encode!(&i, &preprocess).unwrap())
            .call_and_wait()
            .await
            .unwrap();

        println!("{:?}", Decode!(&res));
    }

    // let mut file = File::open("preprocess.bin").unwrap();
    // let mut preprocess = Vec::new();
    // file.read_to_end(&mut preprocess).unwrap();

    // assert_eq!(buffer, preprocess);
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
        .with_arg(Encode!().unwrap())
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
