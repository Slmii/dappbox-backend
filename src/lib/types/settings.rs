use candid::{ CandidType, Deserialize };

#[derive(CandidType, Clone, Deserialize)]
pub struct Settings {
	pub privacy: Privacy,
	pub url: Option<String>,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			privacy: Privacy::Public,
			url: Default::default(),
		}
	}
}

#[derive(CandidType, Clone, Deserialize)]
pub enum Privacy {
	Public,
	Private,
}
