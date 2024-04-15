use candid::{CandidType, Deserialize};
use ic_canister::{generate_idl, init, query, update, Canister, Idl, PreUpdate};
use ic_exports::candid::Principal;
use ic_exports::ic_kit::ic;

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

    #[query]
    pub fn verify_jolt_proof(&self, proof: Vec<u8>) -> Result<bool> {
        ic_exports::ic_cdk::println!("{:?}", proof.len());

        let proof = deserialize_proof(&proof);

        Ok(verify(self.state.preprocess.clone(), proof))
    }

    #[update]
    pub fn preprocessing(&mut self, preprocess: Vec<u8>) -> Result<()> {
        self.state.preprocess = deserialize_preprocessing(&preprocess);
        Ok(())
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
