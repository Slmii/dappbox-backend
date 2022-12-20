use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::{ api::time, caller, id };
use lib::{
	types::{ api_error::{ ApiError, CanisterFailedError }, user::User },
	canister::{ Canister, CanisterSettings, InstallCodeMode, CanisterID },
};
use std::{ cell::RefCell, collections::HashMap };

#[derive(CandidType, Clone, Deserialize, Default)]
pub struct UsersStore {
	pub users: HashMap<Principal, User>,
	pub user_canisters: HashMap<Principal, Vec<Principal>>,
	pub chunks_wasm: Vec<u8>,
}

thread_local! {
	pub static STATE: RefCell<UsersStore> = RefCell::new(UsersStore::default());
}

impl UsersStore {
	// ========== Admin calls

	pub fn get_users() -> Vec<User> {
		STATE.with(|state| state.borrow().users.values().cloned().collect())
	}

	// ========== Non-admin calls

	pub fn get_user(principal: Principal) -> Result<User, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			let opt_user = state.users.get(&principal);
			opt_user.map_or(Err(ApiError::NotFound("USER_NOT_FOUND".to_string())), |user| Ok(user.clone()))
		})
	}

	pub async fn create_user(principal: Principal, username: Option<String>) -> Result<User, ApiError> {
		let user = STATE.with(|state| {
			let mut state = state.borrow_mut();

			if state.users.contains_key(&principal) {
				return Err(ApiError::AlreadyExists("USER_EXISTS".to_string()));
			}

			let user_to_add = User {
				user_id: principal,
				username,
				created_at: time(),
			};

			state.users.insert(principal, user_to_add.clone());

			Ok(user_to_add.clone())
		});

		match user {
			// If user is created
			Ok(user) => {
				// Create new canister
				let created_canister = Self::create_chunks_canister(principal).await;

				match created_canister {
					// If canister is created
					Ok(_) => Ok(user),
					// If not then throw the received error
					Err(err) => Err(err),
				}
			}
			// If not then throw the received error
			Err(error) => Err(error),
		}
	}

	async fn create_chunks_canister(principal: Principal) -> Result<String, ApiError> {
		let canister_settings = CanisterSettings {
			controllers: Some(vec![caller(), id()]),
			compute_allocation: None,
			memory_allocation: None,
			freezing_threshold: None,
		};

		let canister_result = Canister::create(Some(canister_settings), 2_000_000_000_000).await;
		let wasm = STATE.with(|state| state.borrow().chunks_wasm.clone());

		match canister_result {
			// If canister creation is successfull
			Ok(canister) => {
				let wasm_result = canister.install_code(InstallCodeMode::Install, wasm, (Some(principal),)).await;

				// If WASM installation is successfull
				match wasm_result {
					Ok(_) => {
						STATE.with(|state|
							state
								.borrow_mut()
								.user_canisters.entry(principal)
								.or_default()
								.push(CanisterID::from(canister))
						);

						Ok("WASM successfully installed".to_string())
					}
					Err(error) =>
						Err(
							ApiError::CanisterFailed(CanisterFailedError {
								code: error.0,
								message: error.1,
							})
						),
				}
			}
			Err(error) =>
				Err(
					ApiError::CanisterFailed(CanisterFailedError {
						code: error.0,
						message: error.1,
					})
				),
		}
	}
}
