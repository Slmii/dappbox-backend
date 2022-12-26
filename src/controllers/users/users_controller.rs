use std::collections::HashMap;

use crate::users_store::{ UsersStore, STATE };
use candid::{ candid_method, Principal };
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::{ types::{ api_error::ApiError, user::User }, utils::{ validate_anonymous, validate_admin } };

#[pre_upgrade]
fn pre_upgrade() {
	STATE.with(|state| storage::stable_save((state,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
	let (old_store,): (UsersStore,) = storage::stable_restore().unwrap();
	STATE.with(|state| {
		*state.borrow_mut() = old_store;
	});
}

// ========== Admin calls

#[query]
#[candid_method(query)]
fn get_users() -> Result<Vec<User>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(UsersStore::get_users()),
		Err(err) => Err(err),
	}
}

#[query]
#[candid_method(query)]
fn get_user_canisters() -> Result<HashMap<Principal, Vec<Principal>>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(UsersStore::get_user_canisters()),
		Err(err) => Err(err),
	}
}

#[query]
#[candid_method(query)]
fn get_chunks_wasm() -> Vec<u8> {
	match validate_admin(&caller()) {
		Ok(_) => STATE.with(|state| state.borrow().chunks_wasm.clone()),
		Err(_) => vec![],
	}
}

// #[update]
// #[candid_method(update)]
// fn upload_chunks_wasm(chunks_wasm: Vec<u8>) {
// 	STATE.with(|state| {
// 		state.borrow_mut().chunks_wasm = chunks_wasm;
// 	})
// }

// ========== Non-admin calls

#[query]
#[candid_method(query)]
fn get_user() -> Result<User, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => UsersStore::get_user(principal),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
async fn create_user(username: Option<String>) -> Result<User, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => UsersStore::create_user(principal, username).await,
		Err(err) => Err(err),
	}
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "users".to_string());
}
