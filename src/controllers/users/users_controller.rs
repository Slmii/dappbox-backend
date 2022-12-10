use crate::users_store::{ UsersStore, STATE };
use candid::candid_method;
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::types::{ api_error::ApiError, user::User };

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

#[query]
#[candid_method(query)]
fn get_user() -> Result<User, ApiError> {
	UsersStore::get_user(caller())
}

#[query]
#[candid_method(query)]
fn get_users() -> Vec<User> {
	UsersStore::get_users()
}

#[update]
#[candid_method(update)]
fn create_user(username: Option<String>) -> Result<User, ApiError> {
	UsersStore::create_user(caller(), username)
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "users".to_string());
}
