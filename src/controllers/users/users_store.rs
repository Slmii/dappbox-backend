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

	/// Get all users.
	///
	/// # Returns
	/// - `Vec<User>` - Users
	pub fn get_all_users() -> Vec<User> {
		STATE.with(|state| state.borrow().users.values().cloned().collect())
	}

	/// Get all chunk canisters.
	///
	/// # Returns
	/// - `HashMap<Principal, Vec<Principal>>` - Chunk canisters
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

	/// Get user by principal.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	///
	/// # Returns
	/// - `User` - User
	pub fn get_user(caller_principal: Principal) -> Result<User, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			let opt_user = state.users.get(&caller_principal);
			opt_user.map_or(Err(ApiError::NotFound("USER_NOT_FOUND".to_string())), |user| Ok(user.clone()))
		})
	}

	/// Create user.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	/// - `username` - Username
	///
	/// # Returns
	/// - `User` - User
	pub async fn create_user(caller_principal: Principal, username: Option<String>) -> Result<User, ApiError> {
		let user = STATE.with(|state| {
			let mut state = state.borrow_mut();

			if state.users.contains_key(&caller_principal) {
				return Err(ApiError::AlreadyExists("USER_EXISTS".to_string()));
			}

			let user_to_add = User {
				user_id: caller_principal,
				username,
				created_at: time(),
				canisters: vec![],
				alias_user_ids: None,
			};

			state.users.insert(caller_principal, user_to_add.clone());

			Ok(user_to_add.clone())
		});

		match user {
			// If user is created successfully
			Ok(user) => {
				// Create new canister for chunks
				let canister_principal = Self::create_chunks_canister(caller_principal).await;

				match canister_principal {
					// If canister is created successfully
					Ok(canister_principal) => {
						// Add the created canister principal to user field 'canisters'
						STATE.with(|state| {
							let mut state = state.borrow_mut();

							if let Some(user) = state.users.get_mut(&user.user_id) {
								user.canisters.push(canister_principal);
							}

							// Return the created user
							Ok(user)
						})
					}
					// If canister creation failed
					Err(err) => Err(err),
				}
			}
			// If user creation failed
			Err(error) => Err(error),
		}
	}

	/// Create chunks canister.
	/// This canister will be used to store chunks. It will be created for each user.
	///
	/// # Arguments
	/// - `caller_principal` - Principal of the caller
	///
	/// # Returns
	/// - `Principal` - Principal of the created canister
	async fn create_chunks_canister(caller_principal: Principal) -> Result<Principal, ApiError> {
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
				// Install WASM code to the canister
				let wasm_result = canister.install_code(InstallCodeMode::Install, wasm, (
					Some(caller_principal),
				)).await;

				// If WASM installation is successfull
				match wasm_result {
					// Return the principal of the created canister
					Ok(_) => Ok(CanisterID::from(canister)),
					// If WASM installation failed
					Err(error) =>
						Err(
							ApiError::CanisterFailed(CanisterFailedError {
								code: error.0,
								message: error.1,
							})
						),
				}
			}
			// If canister creation failed
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
