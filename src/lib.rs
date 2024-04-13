mod canister;
pub mod error;
pub mod jolt;
pub mod state;

pub use crate::canister::VerifierCanister;

pub fn idl() -> String {
    let idl = VerifierCanister::idl();
    candid::pretty::candid::compile(&idl.env.env, &Some(idl.actor))
}
