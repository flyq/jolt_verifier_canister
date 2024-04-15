use ark_bn254::{Fr, G1Projective};
use ark_serialize::CanonicalDeserialize;
use jolt_core::jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt, JoltPreprocessing};
use jolt_sdk::Proof;

pub fn deserialize_proof(proof: &[u8]) -> Proof {
    Proof::deserialize_compressed(proof).unwrap()
}

pub fn deserialize_preprocessing(preprocessing: &[u8]) -> JoltPreprocessing<Fr, G1Projective> {
    JoltPreprocessing::deserialize_compressed(preprocessing).unwrap()
}

pub fn verify(preprocessing: JoltPreprocessing<Fr, G1Projective>, proof: Proof) -> bool {
    RV32IJoltVM::verify(preprocessing, proof.proof, proof.commitments).is_ok()
}
