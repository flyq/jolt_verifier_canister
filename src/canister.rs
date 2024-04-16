use ark_bn254::{Fr, G1Projective};
use candid::{CandidType, Deserialize};
use ic_canister::{generate_idl, init, query, update, Canister, Idl, PreUpdate};
use ic_exports::candid::Principal;
use ic_exports::ic_kit::ic;
use jolt_core::jolt::vm::JoltPreprocessing;
use jolt_sdk::Proof;
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::error::{Error, Result};
use crate::jolt::{deserialize_preprocessing, deserialize_proof, verify};
use crate::state::{Settings, State};

/// A canister to transfer funds between IC token canisters and EVM canister contracts.
#[derive(Canister)]
pub struct VerifierCanister {
    #[id]
    id: Principal,

    state: State,
}

impl PreUpdate for VerifierCanister {}

impl VerifierCanister {
    /// Initialize the canister with given data.
    #[init]
    pub fn init(&mut self, init_data: InitData) {
        let settings = Settings {
            owner: init_data.owner,
        };

        self.state.reset(settings);
    }

    /// Returns principal of canister owner.
    #[query]
    pub fn get_owner(&self) -> Principal {
        self.state.config.get_owner()
    }

    /// Sets a new principal for canister owner.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    #[update]
    pub fn set_owner(&mut self, owner: Principal) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.config.set_owner(owner)?;
        Ok(())
    }

    #[update]
    pub fn upload_proof_buffer(&mut self, idx: u32, buffer: Vec<u8>) -> Result<()> {
        ic_exports::ic_cdk::println!("proof buffer size: {:?}", buffer.len());

        BUFFER.with(|b| {
            b.borrow_mut().insert(idx, buffer);
        });
        Ok(())
    }

    #[update]
    pub fn update_proof(&mut self, end: u32, func_id: u32) -> Result<u32> {
        BUFFER.with(|b| {
            let mut buffer = Vec::new();

            for idx in 0..=end {
                if b.borrow().get(&idx).is_none() {
                    return Err(Error::Internal(format!("Buffer {} is missing", idx)));
                }
                buffer.extend(b.borrow().get(&idx).unwrap());
            }

            PROOF.with(|p| {
                p.borrow_mut().entry(func_id).or_insert(BTreeMap::new());
                let proof_id = p.borrow_mut().get_mut(&func_id).unwrap().len() as u32;
                p.borrow_mut()
                    .get_mut(&func_id)
                    .unwrap()
                    .insert(proof_id, deserialize_proof(&buffer));
                Ok(proof_id)
            })
        })
    }

    #[update]
    pub fn verify_jolt_proof(&self, func_id: u32, proof_id: u32) -> Result<bool> {
        let preprocess = PREPROCESS
            .with(|p| p.borrow().get(&func_id).unwrap().clone())
            .clone();

        let proof = PROOF.with(|p| {
            p.borrow()
                .get(&func_id)
                .unwrap()
                .get(&proof_id)
                .unwrap()
                .clone()
        });

        Ok(verify(preprocess, proof))
    }

    #[update]
    pub fn upload_preprocessing_buffer(&mut self, idx: u32, preprocess: Vec<u8>) -> Result<()> {
        ic_exports::ic_cdk::println!("preprocessing buffer size: {:?}", preprocess.len());

        BUFFER.with(|buffer| {
            buffer.borrow_mut().insert(idx, preprocess);
        });
        Ok(())
    }

    #[update]
    pub fn preprocessing(&mut self, end: u32, func_id: u32) -> Result<()> {
        BUFFER.with(|b| {
            let mut buffer = Vec::new();

            for idx in 0..=end {
                if b.borrow().get(&idx).is_none() {
                    return Err(Error::Internal(format!("Buffer {} is missing", idx)));
                }
                buffer.extend(b.borrow().get(&idx).unwrap());
            }
            PREPROCESS.with(|p| {
                p.borrow_mut()
                    .insert(func_id, deserialize_preprocessing(&buffer));
            });
            b.borrow_mut().clear();
            Ok(())
        })
    }

    #[update]
    pub fn clear_buffer(&mut self) -> Result<()> {
        BUFFER.with(|b| {
            b.borrow_mut().clear();
        });
        Ok(())
    }

    #[query]
    pub fn get_buffer(&self, idx: u32) -> Option<Vec<u8>> {
        BUFFER.with(|buffer| buffer.borrow().get(&idx).cloned())
    }

    #[query]
    pub fn http_request(&self) -> HttpResponse {
        let body = b"
        This is a Jolt verifier canister.\n
        It has set up the Preprocess of fibonacci, sha2, sha2_chain, sha3, sha3_chain in the Jolt example. \n
        Therefore, it can verify the proof generated by these circuits.\n
        Each program has uploaded 1-2 proofs, so it can also be directly verified through command.\n
        to verify fibonacci(10)'s proof, call:\n
        dfx canister --network ic call verify_jolt_proof '(0:nat32, 0:nat32)'\n
        to verify fibonacci(50)'s proof, call:\n
        dfx canister --network ic call verify_jolt_proof '(0:nat32, 1:nat32)'\n
        to verify sha2's proof, call:\n
        dfx canister --network ic call verify_jolt_proof '(2:nat32, 0:nat32)'\n

        See https://github.com/flyq/jolt_verifier_canister for more details.".to_vec();

        HttpResponse {
            status_code: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            body,
        }
    }

    fn check_owner(&self, principal: Principal) -> Result<()> {
        let owner = self.state.config.get_owner();
        if owner == principal || owner == Principal::anonymous() {
            return Ok(());
        }
        Err(Error::NotAuthorized)
    }

    /// Returns candid IDL.
    /// This should be the last fn to see previous endpoints in macro.
    pub fn idl() -> Idl {
        generate_idl!()
    }
}

/// Minter canister initialization data.
#[derive(Deserialize, CandidType)]
pub struct InitData {
    /// Principal of canister's owner.
    pub owner: Principal,
}

#[derive(CandidType)]
pub struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

// BUFFER used to store the temp preprocessing buffer.
// PREPROCESS used to store the preprocessing struct, function_id => JoltPreprocessing,
// now support 5 function: fibonacci(0), sha2_chain(1), sha2_ex(2), sha3_chain(3), sha3_ex(4)
// PROOF used to store the proof data, function_id => proof_id => Proof
thread_local! {
    static BUFFER: RefCell<BTreeMap<u32, Vec<u8>>> = RefCell::new(BTreeMap::new());
    static PREPROCESS: RefCell<BTreeMap<u32, JoltPreprocessing<Fr, G1Projective>>> = RefCell::new(BTreeMap::new());
    static PROOF: RefCell<BTreeMap<u32, BTreeMap<u32, Proof>>> = RefCell::new(BTreeMap::new());
}
