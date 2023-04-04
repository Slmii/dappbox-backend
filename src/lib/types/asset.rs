use super::{ chunk::Chunk, settings::Settings, nft::Nft };
use candid::{ CandidType, Deserialize, Principal };

#[derive(CandidType, Clone, Deserialize)]
pub struct Asset {
	pub id: u32,
	pub user_id: Principal,
	pub parent_id: Option<u32>,
	pub asset_type: AssetType,
	pub name: String,
	pub is_favorite: bool,
	pub size: u32,
	pub extension: String,
	pub mime_type: String,
	pub created_at: u64,
	pub updated_at: u64,
	pub chunks: Vec<Chunk>,
	pub settings: Settings,
}

impl Default for Asset {
	fn default() -> Self {
		Self {
			id: Default::default(),
			user_id: Principal::anonymous(),
			parent_id: None,
			asset_type: AssetType::Folder,
			name: Default::default(),
			is_favorite: Default::default(),
			size: Default::default(),
			extension: Default::default(),
			mime_type: Default::default(),
			created_at: Default::default(),
			updated_at: Default::default(),
			chunks: Default::default(),
			settings: Default::default(),
		}
	}
}

#[derive(CandidType, Clone, Deserialize)]
pub struct PostAsset {
	pub user_id: Principal,
	pub parent_id: Option<u32>,
	pub asset_type: AssetType,
	pub name: String,
	pub size: u32,
	pub extension: String,
	pub mime_type: String,
	pub chunks: Vec<Chunk>,
	pub settings: Settings,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct EditAsset {
	pub id: u32,
	pub parent_id: Option<u32>,
	pub is_favorite: Option<bool>,
	pub name: Option<String>,
	pub extension: Option<String>,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct MoveAsset {
	pub id: u32,
	pub parent_id: Option<u32>,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct SharedWith {
	pub principal: Principal,
	pub username: Option<String>,
}

#[derive(CandidType, Clone, Deserialize)]
pub enum AssetType {
	Folder,
	File,
	NFT(Nft),
}
