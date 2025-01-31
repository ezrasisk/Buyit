use candid::{CandidType, Deserialize};
use ic_cdk::api::stable::StableReader;
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Entry {
    id: u64,
    text: String,
    image: Vec<u8>,
    user_id: u32,
    creator: Principal,
    status: EntryStatus,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
enum EntryStatus {
    Active,
    Sole,
    Archived,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Database {
    entries: Vec<Entry>,
    next_id: u64,
}


#[update]
fn create_entry(text: String, image: Vec<u8>) -> u64 {
    let caller = ic_cdk::api::caller();
    let mut db = fetch_db().unwrap();
    let next_id = db.next_id;
    db.entries.push(Entry {
        id: new_id,
        text,
        image,
        creator: caller,
        status: EntryStatus::Active,
    });
    db.next_id += 1;
    save_db(db).unwrap();
    new_id
}

fn modify_entry(id: u64, new_text: Option<String>, new_image: Option<vec<u8>>) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let mut db = fetch_db()?
    if let Some(entry) = db.entries.iter_mut().find(|e| e.id == id && e.creator == caller) {
        if let Some(text) = new_text {
            entry.text = text;
        }
        if let Some(image) = new_image {
            entry.image = image;
        }
        save_db(db)?;
        Ok(())
    } else {
        Err("Entry not found or you're not the creator".to_string())
    }
}


#[query]
fn get_entries() -> Vec<Entry> {
    fetch_db().unwrap_or_else(|_| Database{
        entries: vec![], next_id: 0
    }).entries
}


#[query]
fn view_entries() -> Vec<Entry> {
    fetch_db().unwrap().entries
}


#[update]
fn buy_item(entry_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let mut db = fatch_db()?;
    if let Some(entry) = db.entries.iter_mut().find(|e| e.id == entry_id && e.status == EntryStatus::Active) {

    //implement function to verify buying power first, this rn just changes status, obv
    entry.status = EntryStatus::Sold;
    save_db;

    //send reciepts, more complex than this, included later on
    let receipt = format!("Item purchased: Entry ID {}", entry_id);
    send_receipt_to_creator(entry.creator, receipt.clone())?;
    send_receipt_to_buyer(caller, receipt)?;
    Ok(())
    } else {
        Err("Entry not found or not for sale".to_string())
    }
}
fn send_receipt_to_creator(creator: Principal, receipt: String) -> Result<(), String> {
    //implement call to/from another canister
    Ok(())
}
fn send_receipt_to_buyer(buyer: Principal, receipt: String) -> Result<(), String> {
    //similar to above
    Ok(())
}


#[update]
fn update_entry(id: u64, new_text: Option<String>, new_image: Option<Vec<u8>>) -> Result<(), String> {
    let mut db = fetch_db()?;
    if let Some(entry) = db.entries.iter_mut().find(|e| e.id == id) {
        if let Some(text) = new_text {
            entry.text = text;
        }
        if let Some(image) = new_image {
            entry.image = image;
        }
        save_db(db)?;
        Ok(())
    } else {
        Err("Entry not found".to_string())
    }
}


fn fetch_db() -> Result<Database, String> {
    ic_cdk::storage::stable_restore().unwrap_or_else(|_| Database { entries: vec![], next_id: 0 })
}
fn save_db(db: Database) -> Result<(), String> {
    ic_cdk::storage::stable_save((db,)).map_err(|_| "Failed to save database".to_string)
}