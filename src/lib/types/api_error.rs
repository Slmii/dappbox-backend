use candid::Deserialize;
use ic_kit::candid::CandidType;

#[derive(CandidType, Clone, Deserialize)]
pub enum ApiError {
    Unauthorized(String),
    NotFound(String),
    AlreadyExists(String),
}
