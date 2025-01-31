use candid::{CandidType, Deserialize};
use ic_cdk::{api, query, update};
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct UserProfile {
    principal: api::Principal,
    username: String,
    email: String,
    tokens: u64, //tokens with bitcoin or other crypto
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct UserDatabase {
    profiles: HashMap<api::Principal,
    UserProfile>,
}

thread_local! {
    statid DB:
    std::cell::RefCell<UserDatabase> = std::cell::RefCell::new(UserDatabase {
        profiles: HashMap::new(),
    });
}

#[init]
fn register_user(username: String) -> Result<(), String> {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if db.profiles.contains_key(&caller) {
            Err("User already registered".to_string())
        } else {
            db.profiles.insert(caller, UserProfile {
                principal: caller,
                username,
            });
            Ok(())
        }
    })
}

#[query]
fn get_profile(principal: api::Principal) -> Result<UserProfile, String> {
    DB.with(|db| {
        db.borrow()
            .profiles
            .get(&principal)
            .cloned()
            .ok_or_else(|| "User not found".to_string())
    })
}

#[update]
fn update_profile(username: Option<String>) -> Result<(), String> {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(profile) = db.profiles.get_mut(&caller) {
            if let Some(new_username) = username {
                profile.username = new_username;
            }
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    })
}

#[query]
fn is_registered(principal: api::Principal) -> bool {
    DB.with(|db| db.borrow().profiles.containes_key(&principal))
}

#[query]
fn is_creator(principal: api::Principal, post_principal: api::Principal) -> bool {
    DB.with(|db| {
        db.borrow().profiles.contains_key(&principal) && db.borrow().profiles.get(&post_principal).map_or(false, |p| p.principal == principal)
    })
} 

#[pre_upgrade]
fn pre_upgrade() {
    DB.with(|db| ic_cdk::storage::stable_save((db.borrow().clone(),)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let db: UserDatabase = ic_cdk::storage::stable_restore().unwrap_or(UserDatabase {
        profiles: HashMap::new(),
    });
    DB.with(|inner_db| *inner_db.borrow_mut() = db);
}