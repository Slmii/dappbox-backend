use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::{ api::time };
use lib::{
	types::{
		api_error::ApiError,
		asset::{ Asset, EditAsset, PostAsset, AssetType, MoveAsset, SharedWith },
		invite::Invite,
	},
};
use std::{ cell::RefCell, collections::{ HashMap, HashSet } };

#[derive(CandidType, Clone, Deserialize, Default)]
pub struct AssetsStore {
	// Increment of asset IDs
	pub asset_id: u32,
	// All assets
	pub assets: HashMap<u32, Asset>,
	// Caller's assets. Principal = caller, u32 = asset_id
	pub user_assets: HashMap<Principal, Vec<u32>>,
	// Asset invitations. User has invited you to shared his asset
	// Example: User A sends an invite to User B to have acces to User A's asset
	pub asset_invites: HashMap<Principal, Invite>,
	// Shared assets that are not caller's assets, but is granted access to. Principal = caller, u32 = asset_id
	pub shared: HashMap<Principal, Vec<u32>>,
	// List of people that have access to caller's assets. Principal = caller, u32 = asset_id
	pub shared_with: HashMap<(Principal, u32), Vec<SharedWith>>,
}

thread_local! {
	pub static STATE: RefCell<AssetsStore> = RefCell::new(AssetsStore::default());
}

impl AssetsStore {
	// ========== Admin calls

	pub fn get_all_assets() -> Vec<Asset> {
		STATE.with(|state| state.borrow().assets.values().cloned().collect())
	}

	// ========== Non-admin calls

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
				settings: post_asset.settings,
				nft: post_asset.nft,
				created_at: time(),
				updated_at: time(),
			};

			// Add new asset
			state.assets.insert(asset_id, new_asset.clone());
			state.user_assets.entry(principal).or_default().push(asset_id);
			// TODO: loop through principals and add invite to 'asset_invites' -> HashMap<InvitedUserPrincipal, Invite>. If 'InvitedUserPrincipal' exists in HashMap then append new invite

			new_asset
		})
	}

	pub fn edit_asset(principal: Principal, edit_asset: EditAsset) -> Result<Asset, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Find all user_assets linked to the principal (caller)
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

			// Find all user_assets linked to the principal (caller)
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
			let mut state = state.borrow_mut();

			// Find all assets linked to the principal (caller)
			let user_asset_ids = state.user_assets.get(&principal).cloned().unwrap_or_default();

			let source_set: HashSet<u32> = delete_asset_ids.iter().cloned().collect();
			let target_set: HashSet<u32> = user_asset_ids.iter().cloned().collect();

			if !source_set.is_subset(&target_set) {
				return Err(ApiError::NotFound("ASSET_NOT_FOUND".to_string()));
			}

			if let Some(assets) = state.user_assets.get_mut(&principal) {
				assets.retain(|&id| !delete_asset_ids.contains(&id));
			}

			state.assets.retain(|&id, _| !delete_asset_ids.contains(&id));

			Ok(delete_asset_ids)
		})
	}

	// TODO: get_shared_assets(principal) -> exactly the same as 'get_user_assets' but then for shared_assets
	// TODO: get_shared_with(principal, id) -> get a list of people with who my asset is shared with -> have option to invoke
	// TODO: get_invites(principal)
	// TODO: accept_invite(principal, id) -> check if invite exists -> check if invite is expired -> check if asset exists -> check if asset is Privacy::Private -> add asset to 'shared' HashMap -> add user to 'shared_with' HashMap
	// TODO: decline_invite(principal, id) -> check if invite exists -> check if invite is expired -> decline
	// TODO: get_public_asset(id) -> check if asset exists -> check if asset is public -> return asset -> view asset in front-end (in dialog?)
}
