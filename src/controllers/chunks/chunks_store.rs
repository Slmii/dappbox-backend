use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::{ id };
use lib::{ types::{ api_error::ApiError, chunk::{ Chunk, PostChunk } }, whitelist::whitelist };
use std::{ cell::RefCell, collections::HashMap };

#[derive(CandidType, Clone, Deserialize)]
pub struct ChunksStore {
	pub canister_owner: Principal,
	// Increment of chunk IDs
	pub chunk_id: u32,
	// Blobs (u8) of the chunks. u32 = chunk_id
	pub chunks: HashMap<(u32, Principal), Vec<u8>>,
	// TODO: also need a shared chunks?
}

impl Default for ChunksStore {
	fn default() -> Self {
		Self {
			canister_owner: Principal::anonymous(),
			chunk_id: Default::default(),
			chunks: Default::default(),
		}
	}
}

thread_local! {
	pub static STATE: RefCell<ChunksStore> = RefCell::new(ChunksStore::default());
}

impl ChunksStore {
	// Admin call
	pub fn get_chunks(principal: Principal) -> Result<HashMap<(u32, Principal), Vec<u8>>, ApiError> {
		// TODO: implement owner check with given principal after dynamically creating a Chunk canister when creation an account
		if !whitelist().contains(&principal) {
			Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()))
		} else {
			Ok(STATE.with(|state| state.borrow().chunks.clone()))
		}
	}

	pub fn get_chunks_by_chunk_id(chunk_id: u32, principal: Principal) -> Result<Vec<u8>, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			// Get chunks linked to the chunk ID and principal
			let opt_chunks = state.chunks.get(&(chunk_id, principal));

			if let Some(chunks) = opt_chunks {
				Ok(chunks.clone())
			} else {
				Err(ApiError::NotFound("CHUNKS_NOT_FOUND".to_string()))
			}
		})
	}

	pub fn add_chunk(principal: Principal, post_chunk: PostChunk) -> Chunk {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Increment asset chunk ID
			state.chunk_id += 1;
			let chunk_id = state.chunk_id;

			// Add chunk linked to the chunk and principal
			state.chunks.insert((chunk_id, principal), post_chunk.blob);

			Chunk {
				id: chunk_id,
				index: post_chunk.index,
				canister: id(),
			}
		})
	}
}
