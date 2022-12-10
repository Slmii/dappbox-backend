use candid::{ CandidType, Deserialize, Principal };
use ic_cdk::api::time;
use lib::types::{ api_error::ApiError, user::User };
use std::{ cell::RefCell, collections::HashMap };

#[derive(CandidType, Clone, Deserialize, Default)]
pub struct UsersStore {
	pub users: HashMap<Principal, User>,
}

thread_local! {
	pub static STATE: RefCell<UsersStore> = RefCell::new(UsersStore::default());
}

impl UsersStore {
	pub fn get_user(principal: Principal) -> Result<User, ApiError> {
		STATE.with(|state| {
			let state = state.borrow();

			let opt_user = state.users.get(&principal);
			opt_user.map_or(Err(ApiError::NotFound("USER_NOT_FOUND".to_string())), |user| Ok(user.clone()))
		})
	}

	pub fn get_users() -> Vec<User> {
		STATE.with(|state| state.borrow().users.values().cloned().collect())
	}

	pub fn create_user(principal: Principal, username: Option<String>) -> Result<User, ApiError> {
		STATE.with(|state| {
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
		})
	}
}
