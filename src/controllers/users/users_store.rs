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
	pub chunks_wasm: Vec<u8>,
}

thread_local! {
	pub static STATE: RefCell<UsersStore> = RefCell::new(UsersStore::default());
}

impl UsersStore {
	// ========== Admin calls

	pub fn get_all_users() -> Vec<User> {
		STATE.with(|state| state.borrow().users.values().cloned().collect())
	}

	pub fn get_all_chunk_canisters() -> HashMap<Principal, Vec<Principal>> {
		STATE.with(|state| {
			let state = state.borrow();
			let mut result = HashMap::new();

			for (principal, user) in state.users.iter() {
				result.insert(principal.clone(), user.canisters.clone());
			}

			result
		})
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
		STATE.with(|state| {
			let mut state = state.borrow_mut();

			if state.users.contains_key(&principal) {
				return Err(ApiError::AlreadyExists("USER_EXISTS".to_string()));
			}

			let user_to_add = User {
				user_id: principal,
				username,
				created_at: time(),
				canisters: vec![],
			};

			state.users.insert(principal, user_to_add.clone());

			Ok(user_to_add.clone())
		})

		// match user {
		// 	// If user is created
		// 	Ok(user) => {
		// 		// Create new canister
		// 		let canister_principal = Self::create_chunks_canister(principal).await;

		// 		match canister_principal {
		// 			// If canister is created
		// 			Ok(canister_principal) => {
		// 				// Add the created canister principal to user field 'canisters'
		// 				STATE.with(|state| {
		// 					let mut state = state.borrow_mut();

		// 					if let Some(user) = state.users.get_mut(&user.user_id) {
		// 						user.canisters.push(canister_principal);
		// 					}

		// 					Ok(user)
		// 				})
		// 			}
		// 			// If not then throw the received error
		// 			Err(err) => Err(err),
		// 		}
		// 	}
		// 	// If not then throw the received error
		// 	Err(error) => Err(error),
		// }
	}

	async fn create_chunks_canister(principal: Principal) -> Result<Principal, ApiError> {
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
					Ok(_) => Ok(CanisterID::from(canister)),
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
