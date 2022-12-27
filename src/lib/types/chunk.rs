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

#[derive(CandidType)]
pub struct ChunkStoreState {
	pub canister_owner: Principal,
	pub chunk_id: u32,
	pub chunks: Vec<(u32, Principal)>,
}
