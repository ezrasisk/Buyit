use candid::{CandidType, Deserialize};
use ic_cdk::{api, query, update};
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone)]
enum EntryStatus {
    Active,
    Sold,
    Archived
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Entry {
    id: u64,
    text: String,
    image: Vec<u8>,
    creator: api::Principal,
    status: EntryStatus,
}

#[derive(CandidType, Deserialize, Serialize)]
struct Database {
    entries: HashMap<u64, Entry>,
    next_id: u64,
}

//global canister states
thread_local! {
    static DB: std::cell::RefCell<Database> = std::cell::RefCell::new(Database {
        entries: HashMap::new(),
        next_id: 0,
    });
}

#[init]
fn init() {
    //initialize database?
}

#[update]
fn create_entry(text: String, image: Vec<u8>) -> u64 {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let id = db.next_id;
        db.entries.insert(id, Entry {
            id,
            text,
            image,
            creator: caller,
            status: EntryStatus::Active, //perhaps Option\bool<Active, NotActive>()?
        });
        db.next_id += 1;
        id
    })
}

#[query]
fn view_entry(id: u64) -> Result<Entry, String> {
    DB.with(|db| {
        db.borrow()
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| "Entry not found".to_string())
    })
}

#[update]
fn modify_entry(id: u64, new_text: Option<String>, new_image: Option<Vec<u8>>) -> Result<(), String> {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(entry) = db.entries.get_mut(&id) {
            if entry.creator == caller {
                if let Some(text) = new_text {
                    entry.text = text;
                }
                if let Some(image) = new_image {
                    entry.image = image;
                }
                Ok(())
            } else {
                Err("You are not the creator of this entry". to_string())
            } else {
                Err("Entry not found".to_string())
            }
        }
    })
}

#[update]
fn archive_entry(id: u64) -> Result<(), String> {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(entry) = db.entries.get_mut(&id) {
            if entry.creator == caller {
                entry.status = EntryStatus::Archived;
                Ok(())
            } else {
                Err("You are not the creator of this entry".to_string())
            }
        } else {
            Err("Entry not found".to_string())
        }
    })
}

#[update]
fn mark_entry_sold(id: u64) -> Result<(), String> {
    let caller = api::caller();
        DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(entry) = db.entries.get_mut(&id) {
            if entry.status == EntryStatus::Active {
                entry.status = EntryStatus::Sold;
                transaction_canister::process_payment(id, caller);
                receipt_canister::send_receipts(id, entry.creator, caller);
                Ok(())
            } else {
                Err("Entry is not for sale".to_string())
            }
        } else {
            Err("Entry not found".to_string())
        }
    })
}

#[query]
fn view_active_entries() -> Vec<Entry> {
    DB.with(|db| {
        db.borrow().entries.values()
            .filter(|e| e.status == EntryStatus::Active)
            .cloned()
            .collect()
    })
}

#[query]
fn view_all_entries() -> Vec<Entry> {
    DB.with(|db| {
        db.borrow()
            .entries
            .values()
            .cloned()
            .collect()
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    DB.with(|db| ic_cdk::storage::stable_save((db.borrow().clone(),)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let db: Database = ic_cdk::storage::stable_restore().unwrap_or(Database {
        entries: HashMap::new(),
        next_id: 0,
    });
    DB.with(|inner_db| *inner_db.borrow_mut() = db);
}
