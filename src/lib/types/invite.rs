use candid::{ CandidType, Deserialize, Principal };

#[derive(CandidType, Clone, Deserialize)]
pub struct Invite {
	pub invited_by_principal: Principal,
	pub invited_by_username: Option<String>,
	pub asset_id: u32,
	pub status: InviteStatus,
	pub expires_at: Option<u64>,
}

impl Default for Invite {
	fn default() -> Self {
		Self {
			invited_by_principal: Principal::anonymous(),
			invited_by_username: Default::default(),
			asset_id: Default::default(),
			status: InviteStatus::Pending,
			expires_at: Default::default(),
		}
	}
}

#[derive(CandidType, Clone, Deserialize)]
pub enum InviteStatus {
	Accepted,
	Pending,
	Declined,
}
