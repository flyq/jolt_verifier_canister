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
    pub fn update_proof(&mut self, proof: Vec<u8>) -> Result<()> {
        ic_exports::ic_cdk::println!("proof size: {:?}", proof.len());
        PROOF.with(|p| {
            *p.borrow_mut() = Some(deserialize_proof(&proof));
        });
        Ok(())
    }

    #[update]
    pub fn verify_jolt_proof(&self) -> Result<bool> {
        PREPROCESS.with(|preprocess| {
            let preprocess = preprocess.borrow();

            Ok(verify(
                preprocess.clone(),
                PROOF.with(|p| p.borrow().clone().unwrap()),
            ))
        })
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
    pub fn preprocessing(&mut self) -> Result<()> {
        let mut buffer = Vec::new();
        BUFFER.with(|b| {
            for idx in 0..25 {
                if b.borrow().get(&idx).is_none() {
                    return Err(Error::Internal(format!("Buffer {} is missing", idx)));
                }
                buffer.extend(b.borrow().get(&idx).unwrap());
            }
            PREPROCESS.with(|preprocess| {
                *preprocess.borrow_mut() = deserialize_preprocessing(&buffer);
            });
            Ok(())
        })
    }

    #[query]
    pub fn get_buffer(&self, idx: u32) -> Option<Vec<u8>> {
        BUFFER.with(|buffer| buffer.borrow().get(&idx).cloned())
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

thread_local! {
    static BUFFER: RefCell<BTreeMap<u32, Vec<u8>>> = RefCell::new(BTreeMap::new());
    static PREPROCESS: RefCell<JoltPreprocessing<Fr, G1Projective>> = RefCell::new(Default::default());
    static PROOF: RefCell<Option<Proof>> = RefCell::new(Default::default());
}
