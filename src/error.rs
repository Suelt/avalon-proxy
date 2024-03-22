use cosmwasm_std::StdError;
use thiserror::Error;
// Path: src/error.rs

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std (#[from] StdError),
    
}