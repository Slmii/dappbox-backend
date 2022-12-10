use std::collections::HashMap;

use candid::{ candid_method, Principal };
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::types::{ api_error::ApiError, chunk::{ Chunk, PostChunk } };

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

// Admin call
#[query]
#[candid_method(query)]
fn get_chunks() -> Result<HashMap<(u32, Principal), Vec<u8>>, ApiError> {
	ChunksStore::get_chunks(caller())
}

#[query]
#[candid_method(query)]
fn get_chunks_by_chunk_id(chunk_id: u32) -> Result<Vec<u8>, ApiError> {
	ChunksStore::get_chunks_by_chunk_id(chunk_id, caller())
}

#[update]
#[candid_method(update)]
fn add_chunk(chunk: PostChunk) -> Chunk {
	ChunksStore::add_chunk(caller(), chunk)
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "chunks".to_string());
}
