use candid::{ CandidType, Deserialize, Principal };

#[derive(CandidType, Clone, Deserialize)]
pub struct Nft {
	pub principal: Principal,
	pub index: u32,
}

impl Default for Nft {
	fn default() -> Self {
		Self {
			principal: Principal::anonymous(),
			index: Default::default(),
		}
	}
}
