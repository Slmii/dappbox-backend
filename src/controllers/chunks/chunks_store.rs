use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::{ id };
use lib::{ types::{ api_error::ApiError, chunk::{ Chunk, PostChunk } } };
use std::{ cell::RefCell, collections::HashMap };

#[derive(CandidType, Clone, Deserialize)]
pub struct ChunksStore {
	// Caller's principal
	pub canister_owner: Principal,
	// Increment of chunk IDs
	pub chunk_id: u32,
	// Blobs (u8) of the chunks. u32 = chunk_id, Principal = caller
	pub chunks: HashMap<(u32, Principal), Vec<u8>>,
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
	// ========== Admin calls

	pub fn get_chunks() -> HashMap<(u32, Principal), Vec<u8>> {
		STATE.with(|state| state.borrow().chunks.clone())
	}

	// ========== Non-admin calls

	pub fn get_chunks_by_chunk_id(chunk_id: u32, principal: Principal) -> Result<Vec<u8>, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			// Get chunks linked to the chunk ID and principal (caller)
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

			// Add chunk linked to the chunk and principal (caller)
			state.chunks.insert((chunk_id, principal), post_chunk.blob);

			Chunk {
				id: chunk_id,
				index: post_chunk.index,
				canister: id(),
			}
		})
	}
}
