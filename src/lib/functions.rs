use crate::types::asset::{ Asset, AssetType };

pub fn get_child_assets(assets: &Vec<Asset>, parent_id: &u32) -> Vec<Asset> {
	assets
		.iter()
		.filter(|asset| asset.parent_id == Some(*parent_id))
		.cloned()
		.collect()
}

pub fn get_nested_child_assets(assets: &Vec<Asset>, parent_id: &u32) -> Vec<Asset> {
	let child_assets = get_child_assets(assets, parent_id);

	child_assets
		.iter()
		.flat_map(|child_asset| {
			if let AssetType::Folder = child_asset.asset_type {
				get_nested_child_assets(assets, &child_asset.id)
			} else {
				vec![child_asset.clone()]
			}
		})
		.collect()
}
