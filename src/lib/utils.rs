use candid::Principal;
use crate::{ types::{ asset::{ Asset, AssetType }, api_error::ApiError }, whitelist::whitelist };

#[allow(dead_code)]
const WASM_PAGE_SIZE: u64 = 65536;

pub fn get_nested_child_assets(assets: &Vec<Asset>, asset_id: &u32) -> Vec<u32> {
	let mut child_assets: Vec<u32> = vec![];

	for asset in assets {
		if asset.parent_id == Some(*asset_id) {
			child_assets.push(asset.id);

			if let AssetType::Folder = asset.asset_type {
				let nested_child_assets = get_nested_child_assets(assets, &asset.id);
				child_assets.extend(nested_child_assets);
			}
		}
	}

	child_assets
}

pub fn validate_anonymous(principal: &Principal) -> Result<Principal, ApiError> {
	Principal::from_text("2vxsx-fae").map_or(Err(ApiError::Unauthorized("UNAUTHORIZED".to_string())), |anon_principal| {
		if *principal == anon_principal {
			return Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()));
		}

		return Ok(*principal);
	})
}

pub fn validate_admin(principal: &Principal) -> Result<Principal, ApiError> {
	if !whitelist().contains(&principal) {
		return Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()));
	}

	Ok(*principal)
}

pub fn validate_anonymous_and_admin(principal: &Principal) -> Result<Principal, ApiError> {
	validate_anonymous(principal)?;
	validate_admin(principal)?;

	Ok(*principal)
}

pub fn get_cycles() -> u64 {
	#[cfg(target_arch = "wasm32")]
	{
		ic_cdk::api::canister_balance()
	}
	#[cfg(not(target_arch = "wasm32"))]
	{
		0
	}
}

pub fn get_heap_memory_size() -> u64 {
	#[cfg(target_arch = "wasm32")]
	{
		(core::arch::wasm32::memory_size(0) as u64) * WASM_PAGE_SIZE
	}
	#[cfg(not(target_arch = "wasm32"))]
	{
		0
	}
}
