use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::api::{ time, call };
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

	/// Get all assets.
	///
	/// # Returns
	/// - `Vec<Asset>` - Assets
	pub fn get_all_assets() -> Vec<Asset> {
		STATE.with(|state| state.borrow().assets.values().cloned().collect())
	}

	// ========== Non-admin calls

	/// Get assets by principal.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	///
	/// # Returns
	/// - `Vec<Asset>` - Assets
	pub fn get_user_assets(caller_principal: Principal) -> Vec<Asset> {
		STATE.with(|state| {
			let state = state.borrow();

			// Get user's assets
			let user_asset_ids_by_principal = state.user_assets.get(&caller_principal).cloned().unwrap_or_default();

			// Loop through all assets and check if the asset_id contains in user's assets list
			state.assets
				.values()
				.filter(|asset| user_asset_ids_by_principal.contains(&asset.id))
				.cloned()
				.collect()
		})
	}

	/// Add asset.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `post_asset` - Asset to add
	///
	/// # Returns
	/// - `Asset` - Added asset
	pub async fn add_asset(caller_principal: Principal, post_asset: PostAsset) -> Asset {
		// If existing asset, then do intercanister call to previous delete_chunks
		if let Some(asset_id) = post_asset.id {
			// Get user assets using get_user_assets call
			let user_assets = Self::get_user_assets(caller_principal.clone());

			// Find a specific asset by id
			let asset = user_assets.into_iter().find(|asset| asset.id == asset_id);

			if let Some(asset) = asset {
				// Get chunks from asset
				let chunks = asset.chunks;

				// Get all chunk ids
				let chunk_ids: Vec<u32> = chunks
					.iter()
					.map(|chunk| chunk.id)
					.collect();

				// Get first chunk's canister principal
				let caniser_principal = chunks.first().unwrap().canister;

				let _: Result<(Result<Vec<u32>, ApiError>,), _> = call::call(
					caniser_principal.clone(),
					"delete_chunks_intercanister_call",
					(caller_principal.clone(), chunk_ids.clone())
				).await;
			}
		}

		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Find all user_assets linked to the principal (caller)
			let user_asset_ids = state.user_assets.get(&caller_principal).cloned().unwrap_or_default();
			// Find a specific asset with given value
			let asset_id = user_asset_ids
				.into_iter()
				.find(|&asset_id| post_asset.id.filter(|id| *id == asset_id).is_some());

			// TODO: loop through principals and add invite to 'asset_invites' -> HashMap<InvitedUserPrincipal, Invite>. If 'InvitedUserPrincipal' exists in HashMap then append new invite

			asset_id
				.and_then(|asset_id| state.assets.get_mut(&asset_id))
				.map(|found_asset| {
					// Mutate values
					found_asset.size = post_asset.size;
					found_asset.updated_at = time();

					found_asset.clone()
				})
				.unwrap_or_else(|| {
					// Increment asset ID
					state.asset_id += 1;
					let asset_id = state.asset_id;

					let new_asset = Asset {
						id: asset_id,
						user_id: caller_principal,
						parent_id: post_asset.parent_id,
						asset_type: post_asset.asset_type,
						name: post_asset.name,
						is_favorite: false,
						size: post_asset.size,
						extension: post_asset.extension,
						mime_type: post_asset.mime_type,
						chunks: post_asset.chunks,
						settings: post_asset.settings,
						created_at: time(),
						updated_at: time(),
					};

					// Add new asset or overwrite existing one
					state.assets.insert(asset_id, new_asset.clone());

					// Add asset to user_assets
					state.user_assets.entry(caller_principal).or_default().push(asset_id);

					new_asset
				})
		})
	}

	/// Edit asset.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `edit_asset` - Asset to edit
	///
	/// # Returns
	/// - `Asset` - Edited asset
	pub fn edit_asset(caller_principal: Principal, edit_asset: EditAsset) -> Result<Asset, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Find all user_assets linked to the principal (caller)
			let user_asset_ids = state.user_assets.get(&caller_principal).cloned().unwrap_or_default();
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
						AssetType::NFT(_) => {
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

	/// Move assets to different parent.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `move_assets` - Assets to move
	///
	/// # Returns
	/// - `Vec<Asset>` - Moved assets
	pub fn move_assets(caller_principal: Principal, move_assets: Vec<MoveAsset>) -> Result<Vec<Asset>, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();
			let mut temp: Vec<Asset> = vec![];

			// Find all user_assets linked to the principal (caller)
			let user_asset_ids = state.user_assets.get(&caller_principal).cloned().unwrap_or_default();

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

	/// Delete assets.
	/// If the asset is a folder, all children will be deleted as well.
	/// If the asset is a file, only the file will be deleted.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `delete_asset_ids` - Asset IDs to delete
	///
	/// # Returns
	/// - `Vec<u32>` - Deleted asset IDs
	pub fn delete_assets(caller_principal: Principal, delete_asset_ids: Vec<u32>) -> Result<Vec<u32>, ApiError> {
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			// Find all assets linked to the principal (caller)
			let user_asset_ids = state.user_assets.get(&caller_principal).cloned().unwrap_or_default();

			let source_set: HashSet<u32> = delete_asset_ids.iter().cloned().collect();
			let target_set: HashSet<u32> = user_asset_ids.iter().cloned().collect();

			if !source_set.is_subset(&target_set) {
				return Err(ApiError::NotFound("ASSET_NOT_FOUND".to_string()));
			}

			if let Some(assets) = state.user_assets.get_mut(&caller_principal) {
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
