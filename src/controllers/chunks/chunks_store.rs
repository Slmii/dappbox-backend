use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::id;
use lib::{ types::{ api_error::ApiError, chunk::{ Chunk, PostChunk } }, utils::validate_admin };
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

	/// Get all chunks.
	///
	/// # Returns
	/// - `HashMap<(u32, Principal), Vec<u8>>` - Chunks
	pub fn get_all_chunks() -> HashMap<(u32, Principal), Vec<u8>> {
		STATE.with(|state| state.borrow().chunks.clone())
	}

	// ========== Non-admin calls

	/// Get chunks by chunk ID.
	///
	/// # Arguments
	/// - `chunk_id` - Chunk ID
	/// - `caller_principal` - Principal of the caller
	///
	/// # Returns
	/// - `Vec<u8>` - Chunks
	pub fn get_chunks_by_chunk_id(chunk_id: u32, caller_principal: Principal) -> Result<Vec<u8>, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			if validate_admin(&caller_principal).is_ok() {
				// If the canister owner is not the anonymous `chunks` canister, return an error.
				// Admin can only delete chunks from the anonymous `chunks` canister.
				if state.canister_owner != Principal::from_text("2vxsx-fae").unwrap() {
					return Err(ApiError::NotFound("CHUNKS_NOT_FOUND".to_string()));
				}
			} else if caller_principal != state.canister_owner {
				// If the caller is not the canister owner, return an error
				return Err(ApiError::NotFound("CHUNKS_NOT_FOUND".to_string()));
			}

			// Get chunks linked to the chunk ID and principal (caller)
			let opt_chunks = state.chunks.get(&(chunk_id, caller_principal));

			if let Some(chunks) = opt_chunks {
				Ok(chunks.clone())
			} else {
				Err(ApiError::NotFound("CHUNKS_NOT_FOUND".to_string()))
			}
		})
	}

	/// Add a chunk.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `post_chunk` - Chunk to add
	///
	/// # Returns
	/// - `Chunk` - Chunk added
	pub fn add_chunk(caller_principal: Principal, post_chunk: PostChunk) -> Result<Chunk, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			if validate_admin(&caller_principal).is_ok() {
				// If the canister owner is not the anonymous `chunks` canister, return an error.
				// Admin can only delete chunks from the anonymous `chunks` canister.
				if state.canister_owner != Principal::from_text("2vxsx-fae").unwrap() {
					return Err(ApiError::NotFound("UNAUTHORIZED".to_string()));
				}
			} else if caller_principal != state.canister_owner {
				// If the caller is not the canister owner, return an error
				return Err(ApiError::NotFound("UNAUTHORIZED".to_string()));
			}

			// Increment asset chunk ID
			state.chunk_id += 1;
			let chunk_id = state.chunk_id;

			// Add chunk linked to the chunk and principal (caller)
			state.chunks.insert((chunk_id, caller_principal), post_chunk.blob);

			Ok(Chunk {
				id: chunk_id,
				index: post_chunk.index,
				canister: id(),
			})
		})
	}

	/// Delete chunks.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `delete_chunk_ids` - Chunk IDs to delete
	///
	/// # Returns
	/// - `Vec<u32>` - Chunk IDs that were deleted
	pub fn delete_chunks(caller_principal: Principal, delete_chunk_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();
			let mut removed_chunk_ids = Vec::new();

			if validate_admin(&caller_principal).is_ok() {
				// If the canister owner is not the anonymous `chunks` canister, return an error.
				// Admin can only delete chunks from the anonymous `chunks` canister.
				if state.canister_owner != Principal::from_text("2vxsx-fae").unwrap() {
					return Err(ApiError::NotFound("UNAUTHORIZED".to_string()));
				}
			} else if caller_principal != state.canister_owner {
				// If the caller is not the canister owner, return an error
				return Err(ApiError::NotFound("UNAUTHORIZED".to_string()));
			}

			for id in delete_chunk_ids {
				if state.chunks.remove(&(id, caller_principal)).is_some() {
					removed_chunk_ids.push(id);
				}
			}

			Ok(removed_chunk_ids)
		})
	}

	/// Delete chunks. This should only be called by the `assets` canister to delete old chunks when
	/// uploading the exact same asset.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `delete_chunk_ids` - Chunk IDs to delete
	///
	/// # Returns
	/// - `Vec<u32>` - Chunk IDs that were deleted
	pub fn delete_chunks_intercanister_call(
		caller_principal: Principal,
		delete_chunk_ids: Vec<u32>
	) -> Result<Vec<u32>, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();
			let mut removed_chunk_ids = Vec::new();

			// Delete chunks linked to the chunk IDs and principal (caller)
			for id in delete_chunk_ids {
				if state.chunks.remove(&(id, caller_principal)).is_some() {
					removed_chunk_ids.push(id);
				}
			}

			Ok(removed_chunk_ids)
		})
	}
}
