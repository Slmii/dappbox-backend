use crate::assets_store::{ AssetsStore, STATE };
use candid::{ candid_method };
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::{
	types::{ api_error::ApiError, asset::{ Asset, PostAsset, EditAsset, MoveAsset } },
	utils::{ validate_anonymous, validate_admin },
};

#[pre_upgrade]
fn pre_upgrade() {
	STATE.with(|state| storage::stable_save((state,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
	let (old_store,): (AssetsStore,) = storage::stable_restore().unwrap();
	STATE.with(|state| {
		*state.borrow_mut() = old_store;
	});
}

// ========== Admin calls

#[query]
#[candid_method(query)]
fn get_state() -> Result<AssetsStore, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(STATE.with(|state| state.borrow().clone())),
		Err(err) => Err(err),
	}
}

#[query]
#[candid_method(query)]
fn get_all_assets() -> Result<Vec<Asset>, ApiError> {
	match validate_admin(&caller()) {
		Ok(_) => Ok(AssetsStore::get_all_assets()),
		Err(err) => Err(err),
	}
}

// ========== Non-admin calls

#[query]
#[candid_method(query)]
fn get_user_assets() -> Result<Vec<Asset>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => Ok(AssetsStore::get_user_assets(principal)),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn add_asset(asset: PostAsset) -> Result<Asset, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => Ok(AssetsStore::add_asset(principal, asset)),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn edit_asset(asset: EditAsset) -> Result<Asset, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => AssetsStore::edit_asset(principal, asset),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn move_assets(assets: Vec<MoveAsset>) -> Result<Vec<Asset>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => AssetsStore::move_assets(principal, assets),
		Err(err) => Err(err),
	}
}

#[update]
#[candid_method(update)]
fn delete_assets(asset_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
	match validate_anonymous(&caller()) {
		Ok(principal) => AssetsStore::delete_assets(principal, asset_ids),
		Err(err) => Err(err),
	}
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "assets".to_string());
}
