use std::collections::HashMap;

use candid::{ candid_method, Principal };
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update, init };
use lib::{
	types::{ api_error::{ ApiError }, chunk::{ Chunk, PostChunk, ChunkStoreState } },
	utils::{ validate_anonymous, validate_admin },
};
use crate::chunks_store::{ ChunksStore, STATE };

#[pre_upgrade]
fn pre_upgrade() {
	STATE.with(|state| storage::stable_save((state,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
	let (old_store,): (ChunksStore,) = storage::stable_restore().unwrap();
	STATE.with(|state| {
		*state.borrow_mut() = old_store;
	});
}

// ========== Admin calls

#[query]
#[candid_method(query)]
fn get_state() -> Result<ChunkStoreState, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) =>
			Ok(
				STATE.with(|state| {
					let state = state.borrow();

					ChunkStoreState {
						canister_owner: state.canister_owner,
						chunk_id: state.chunk_id,
						chunks: state.chunks.keys().cloned().collect(),
					}
				})
			),
		Err(err) => Err(err),
	}
}

#[query]
#[candid_method(query)]
fn get_all_chunks() -> Result<HashMap<(u32, Principal), Vec<u8>>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(ChunksStore::get_all_chunks()),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn delete_chunks_intercanister_call(caller_principal: Principal, chunk_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => ChunksStore::delete_chunks_intercanister_call(caller_principal, chunk_ids),
		Err(err) => Err(err),
	}
}

// ========== Non-admin calls

#[query]
#[candid_method(query)]
fn get_chunks_by_chunk_id(chunk_id: u32) -> Result<Vec<u8>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(caller_principal) => ChunksStore::get_chunks_by_chunk_id(chunk_id, caller_principal),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn add_chunk(chunk: PostChunk) -> Result<Chunk, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(caller_principal) => ChunksStore::add_chunk(caller_principal, chunk),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn delete_chunks(chunk_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(caller_principal) => ChunksStore::delete_chunks(caller_principal, chunk_ids),
		Err(err) => Err(err),
	}
}

#[init]
#[candid_method(init)]
fn init(canister_owner: Option<Principal>) {
	STATE.with(|state| {
		if let Some(owner) = canister_owner {
			state.borrow_mut().canister_owner = owner;
		}
	});
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "chunks".to_string());
}
