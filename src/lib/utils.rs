use candid::Principal;
use crate::{ types::{ asset::{ Asset, AssetType }, api_error::ApiError }, whitelist::whitelist };

/// Get nested child assets.
///
/// # Arguments
/// - `assets` - Assets
/// - `asset_id` - Asset ID
///
/// # Returns
/// - `Vec<u32>` - Child assets
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

/// Validate anonymous.
///
/// # Arguments
/// - `principal` - Principal
///
/// # Returns
/// - `Result<Principal, ApiError>` - Principal or ApiError
pub fn validate_anonymous(principal: &Principal) -> Result<Principal, ApiError> {
	Principal::from_text("2vxsx-fae").map_or(Err(ApiError::Unauthorized("UNAUTHORIZED".to_string())), |anon_principal| {
		if *principal == anon_principal {
			return Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()));
		}

		return Ok(*principal);
	})
}

/// Validate admin.
///
/// # Arguments
/// - `principal` - Principal
///
/// # Returns
/// - `Result<Principal, ApiError>` - Principal or ApiError
pub fn validate_admin(principal: &Principal) -> Result<Principal, ApiError> {
	if !whitelist().contains(&principal) {
		return Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()));
	}

	Ok(*principal)
}

/// Validate anonymous and admin.
///
/// # Arguments
/// - `principal` - Principal
///
/// # Returns
/// - `Result<Principal, ApiError>` - Principal or ApiError
pub fn validate_anonymous_and_admin(principal: &Principal) -> Result<Principal, ApiError> {
	validate_anonymous(principal)?;
	validate_admin(principal)?;

	Ok(*principal)
}
