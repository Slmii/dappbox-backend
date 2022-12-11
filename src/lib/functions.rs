use crate::types::asset::{ Asset, AssetType };

pub fn get_nested_child_assets(assets: &Vec<Asset>, asset_id: &u32) -> Vec<u32> {
	let mut child_assets: Vec<u32> = vec![];

	for asset in assets {
		if asset.parent_id == Some(*asset_id) {
			child_assets.push(asset.id);

			if let AssetType::Folder = asset.asset_type {
				let nested_child_assets = get_nested_child_assets(assets, &asset_id);
				child_assets.extend(nested_child_assets);
			}
		}
	}

	child_assets
}
