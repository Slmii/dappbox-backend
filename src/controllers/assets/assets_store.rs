use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::{ api::time };
use lib::{
	types::{ api_error::ApiError, asset::{ Asset, EditAsset, PostAsset, AssetType, MoveAsset } },
	functions::{ get_nested_child_assets },
	whitelist::whitelist,
};
use std::{ cell::RefCell, collections::HashMap };

#[derive(CandidType, Clone, Deserialize, Default)]
pub struct AssetsStore {
	// Increment of asset IDs
	pub asset_id: u32,
	// All assets
	pub assets: HashMap<u32, Asset>,
	// User assets. u32 = asset_id
	pub user_assets: HashMap<Principal, Vec<u32>>,
	// // Shared assets
	// pub shared_assets: HashMap<Principal, Vec<u32>>,
	// // Shared assets with users
	// pub shared_assets_with: HashMap<u32, Vec<(Principal, String)>>,
}

thread_local! {
	pub static STATE: RefCell<AssetsStore> = RefCell::new(AssetsStore::default());
}

impl AssetsStore {
	// Admin call
	pub fn get_assets(principal: Principal) -> Result<Vec<Asset>, ApiError> {
		if !whitelist().contains(&principal) {
			return Err(ApiError::Unauthorized("UNAUTHORIZED".to_string()));
		}

		Ok(STATE.with(|state| state.borrow().assets.values().cloned().collect()))
	}

	pub fn get_user_assets(principal: Principal) -> Vec<Asset> {
		STATE.with(|state| {
			let state = state.borrow();

			// Get user's assets
			let user_asset_ids_by_principal = state.user_assets.get(&principal).cloned().unwrap_or_default();

			// Loop through all assets and check if the asset_id contains in user's assets list
			state.assets
				.values()
				.filter(|asset| user_asset_ids_by_principal.contains(&asset.id))
				.cloned()
				.collect()
		})
	}

	pub fn add_asset(principal: Principal, post_asset: PostAsset) -> Asset {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Increment asset ID
			state.asset_id += 1;
			let asset_id = state.asset_id;

			let new_asset = Asset {
				id: asset_id,
				user_id: principal,
				parent_id: post_asset.parent_id,
				asset_type: post_asset.asset_type,
				name: post_asset.name,
				is_favorite: false,
				size: post_asset.size,
				extension: post_asset.extension,
				mime_type: post_asset.mime_type,
				chunks: post_asset.chunks,
				created_at: time(),
				updated_at: time(),
			};

			// Add new asset
			state.assets.insert(asset_id, new_asset.clone());
			state.user_assets.entry(principal).or_default().push(asset_id);

			new_asset
		})
	}

	pub fn edit_asset(principal: Principal, edit_asset: EditAsset) -> Result<Asset, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Find all user_assets linked to the principal
			let user_asset_ids = state.user_assets.get(&principal).cloned().unwrap_or_default();
			// Find a specific asset with given value
			let asset_id = user_asset_ids.into_iter().find(|&asset_id| asset_id == edit_asset.id);

			asset_id
				.and_then(|asset_id| state.assets.get_mut(&asset_id))
				.map(|found_asset| {
					// Mutate values
					found_asset.parent_id = edit_asset.parent_id;

					if let Some(name) = edit_asset.name {
						found_asset.name = name;
					}

					match found_asset.asset_type {
						AssetType::File => {
							if let Some(extension) = edit_asset.extension {
								found_asset.extension = extension;
							}
						}
						AssetType::Folder => {
							found_asset.extension = "".to_string();
						}
					}

					if let Some(is_favorite) = edit_asset.is_favorite {
						found_asset.is_favorite = is_favorite;
					}

					found_asset.updated_at = time();

					found_asset.clone()
				})
				.ok_or(ApiError::NotFound("ASSET_NOT_FOUND".to_string()))
		})
	}

	pub fn move_assets(principal: Principal, move_assets: Vec<MoveAsset>) -> Result<Vec<Asset>, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();
			let mut temp: Vec<Asset> = vec![];

			// Find all user_assets linked to the principal
			let user_asset_ids = state.user_assets.get(&principal).cloned().unwrap_or_default();

			for move_asset in move_assets {
				// Find a specific asset based on the asset to move
				let asset_id = user_asset_ids
					.clone()
					.into_iter()
					.find(|&asset_id| asset_id == move_asset.id);

				let asset = asset_id
					.and_then(|asset_id| state.assets.get_mut(&asset_id))
					.map(|found_asset| {
						// Mutate values
						found_asset.parent_id = move_asset.parent_id;
						found_asset.updated_at = time();

						found_asset.clone()
					})
					.ok_or(ApiError::NotFound("ASSET_NOT_FOUND".to_string()))?;

				temp.push(asset.clone());
			}

			Ok(temp)
		})
	}

	pub fn delete_assets(principal: Principal, delete_asset_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
		STATE.with(|state| {
			let user_assets = Self::get_user_assets(principal);

			let mut temp: Vec<u32> = vec![];

			for delete_asset_id in delete_asset_ids {
				let mut state = state.borrow_mut();

				// Find all assets linked to the principal
				let user_asset_ids = state.user_assets.get(&principal).cloned().unwrap_or_default();

				// Check if the Vec has the asset_id that must be removed
				if !user_asset_ids.contains(&delete_asset_id) {
					return Err(ApiError::NotFound("ASSET_NOT_FOUND".to_string()));
				}

				// Get all child + nested assets that will be deleted
				let mut assets_to_delete = get_nested_child_assets(&user_assets, &delete_asset_id);
				// Include the asset_id for the for loop
				assets_to_delete.push(delete_asset_id);

				// Retain/keep if the current id is not included in the assets_to_delete list
				state.user_assets
					.get_mut(&principal)
					.cloned()
					.unwrap_or_default()
					.retain(|&id| !assets_to_delete.contains(&id));
				state.assets.retain(|&id, _| !assets_to_delete.contains(&id));
				// TODO: also delete chunks

				temp.extend(assets_to_delete);
			}

			Ok(temp)
		})
	}
}
