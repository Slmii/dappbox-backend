use crate::assets_store::{ AssetsStore, STATE };
use candid::candid_method;
use ic_cdk::{ caller, storage };
use ic_cdk_macros::{ post_upgrade, pre_upgrade, query, update };
use lib::types::{ api_error::ApiError, asset::{ Asset, PostAsset, EditAsset, MoveAsset } };

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

// Admin call
#[query]
#[candid_method(query)]
fn get_assets() -> Result<Vec<Asset>, ApiError> {
	AssetsStore::get_assets(caller())
}

#[query]
#[candid_method(query)]
fn get_user_assets() -> Vec<Asset> {
	AssetsStore::get_user_assets(caller())
}

#[update]
#[candid_method(update)]
fn add_asset(asset: PostAsset) -> Asset {
	AssetsStore::add_asset(caller(), asset)
}

#[update]
#[candid_method(update)]
fn edit_asset(asset: EditAsset) -> Result<Asset, ApiError> {
	AssetsStore::edit_asset(caller(), asset)
}

#[update]
#[candid_method(update)]
fn move_assets(assets: Vec<MoveAsset>) -> Result<Vec<Asset>, ApiError> {
	AssetsStore::move_assets(caller(), assets)
}

#[update]
#[candid_method(update)]
fn delete_assets(asset_ids: Vec<u32>) -> Result<Vec<Asset>, ApiError> {
	AssetsStore::delete_assets(caller(), asset_ids)
}

#[test]
fn generate_candid() {
	use candid::export_service;
	use lib::save_candid;
	export_service!();

	save_candid::save_candid(__export_service(), "assets".to_string());
}
