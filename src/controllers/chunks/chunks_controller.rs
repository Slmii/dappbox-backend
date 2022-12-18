use std::collections::HashMap;

use candid::{ candid_method, Principal };
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::{ types::{ api_error::ApiError, chunk::{ Chunk, PostChunk } }, utils::{ validate_anonymous, validate_admin } };

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
fn get_chunks() -> Result<HashMap<(u32, Principal), Vec<u8>>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(ChunksStore::get_chunks()),
		Err(err) => Err(err),
	}
}

// ========== Non-admin calls

#[query]
#[candid_method(query)]
fn get_chunks_by_chunk_id(chunk_id: u32) -> Result<Vec<u8>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => ChunksStore::get_chunks_by_chunk_id(chunk_id, principal),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn add_chunk(chunk: PostChunk) -> Result<Chunk, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => Ok(ChunksStore::add_chunk(principal, chunk)),
		Err(err) => Err(err),
	}
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "chunks".to_string());
}
