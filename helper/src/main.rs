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
        "generate_preprocess" => generate_preprocess(args[2].as_str(), args[3].as_str()),
        "check_split" => check_split(),
        "upload_preprocess" => upload_preprocess().await,
        "call_preprocess" => call_preprocess().await,
        "upload_proof" => upload_proof().await,
        "update_proof" => update_proof().await,
        _ => panic!("Invalid command"),
    }
}

fn generate_preprocess(guest: &str, func: &str) {
    let mut program = Program::new(guest);
    program.set_func(func);

    let target = format!("/tmp/jolt-guest-target-{}-{}", guest, func);

    let elf = format!("{}/riscv32i-unknown-none-elf/release/guest", target,);

    program.elf = Some(PathBuf::from_str(&elf).unwrap());

    let (bytecode, memory_init) = program.decode();

    let preprocessing: JoltPreprocessing<Fr, G1Projective> =
        RV32IJoltVM::preprocess(bytecode, memory_init, 1 << 10, 1 << 10, 1 << 14);

    let mut buffer: Vec<u8> = Vec::new();
    preprocessing.serialize_compressed(&mut buffer).unwrap();
    println!("buffer size: {}", buffer.len());

    let total = buffer.len() / 2_000_000 + 1;
    for i in 0..total {
        let name = format!("data/{}/p{}.bin", func, i);
        let mut file = File::create(name).unwrap();

        if i < total - 1 {
            let temp = &buffer[i * 2_000_000..(i + 1) * 2_000_000];
            // write temp to file
            file.write_all(temp).unwrap();
        } else {
            let temp = &buffer[i * 2_000_000..];
            // write temp to file
            file.write_all(temp).unwrap();
        }
    }

    let name = format!("data/{}/preprocess.bin", func);

    let file = File::create(name).unwrap();
    preprocessing.serialize_compressed(file).unwrap();
}

fn check_split() {
    let mut file = File::open("data/sha2_chain/sha2_chain100.bin").unwrap();
    let mut proof = Vec::new();
    file.read_to_end(&mut proof).unwrap();

    let mut file = File::create("data/sha2_chain/sha2_chain100_0.bin").unwrap();
    file.write_all(&proof[..2_000_000]).unwrap();

    let mut file = File::create("data/sha2_chain/sha2_chain100_1.bin").unwrap();
    file.write_all(&proof[2_000_000..]).unwrap();

    // let mut buffer: Vec<u8> = Vec::new();

    // for i in 0..25 {
    //     let name = format!("data/fib/p{}.bin", i);
    //     let mut file = File::open(name).unwrap();
    //     let mut temp = Vec::new();
    //     file.read_to_end(&mut temp).unwrap();
    //     buffer.extend(temp);
    // }

    // assert_eq!(preprocess10, buffer);

    println!("successfully");
}

async fn upload_preprocess() {
    let agent = Agent::builder()
        // .with_url("http://localhost:4943")
        .with_url("https://ic0.app")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    // let canister_id: Principal = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();
    let canister_id = Principal::from_text("p6xvw-7iaaa-aaaap-aaana-cai").unwrap();

    for i in 0..25u32 {
        let name = format!("data/fib/p{}.bin", i);
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

async fn upload_proof() {
    let agent = Agent::builder()
        // .with_url("http://localhost:4943")
        .with_url("https://ic0.app")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    // let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();
    let canister_id = Principal::from_text("p6xvw-7iaaa-aaaap-aaana-cai").unwrap();

    for i in 0..1u32 {
        let name = format!("data/fib/fib50.bin");
        let mut file = File::open(name).unwrap();
        let mut temp = Vec::new();
        file.read_to_end(&mut temp).unwrap();

        println!("{}th, part preprocess size: {}", i, temp.len());

        let res = agent
            .update(&canister_id, "upload_proof_buffer")
            .with_arg(Encode!(&i, &temp).unwrap())
            .call_and_wait()
            .await
            .unwrap();

        println!("{:?}", Decode!(&res));
    }
}

async fn update_proof() {
    let agent = Agent::builder()
        .with_url("http://localhost:4943")
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();

    let canister_id = Principal::from_text("bnz7o-iuaaa-aaaaa-qaaaa-cai").unwrap();

    let res = agent
        .update(&canister_id, "update_proof")
        .with_arg(Encode!(&0u32, &0u32).unwrap())
        .call_and_wait()
        .await
        .unwrap();

    println!("{:?}", Decode!(&res));
}
