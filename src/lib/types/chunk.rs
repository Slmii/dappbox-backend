use candid::{ CandidType, Deserialize, Principal };

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Chunk {
	pub id: u32,
	pub index: u32,
	pub canister: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PostChunk {
	pub blob: Vec<u8>,
	pub index: u32,
}
