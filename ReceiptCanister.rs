use candid::{CandidType, Deserialize};
use ic_cdk::{api, update, query};
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::call;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Receipt {
    id: u64,
    transaction_id: u64,
    buyer: api::Principal,
    seller: api::Principal,
    amount: u64,
    item_id: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
struct ReceiptDatabase {
    receipts: HashMap<u64, Receipt>,
    next_receipt_id: u64,
}

thread_local! {
    static DB:
    std::cell::RefCell<ReceiptDatabase> = std::cell::RefCell::new(ReceiptDatabase {
        receipts: HashMap::new(),
        next_receipt_id: 0,
    });
}

#[init]
fn init() {
    //initialize databse?
}

#[update]
fn generate_recepit(transaction_id: u64, buyer: api::Principal, seller: api::Principal, amount: u64, item_id: u64) -> u64 {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let receipt_id = db.next_receipt_id;
        db.receipt.insert(receipt_id, Receipt {
            id: receipt_id,
            transaction_id,
            buyer,
            seller,
            amount,
            item_id,
        });
        db.next_receipt_id += 1;
        receipt_id
    })
}

#[update]
async fn send_receipts(receipt_id: u64) -> Result<(), String> {
    DB.with(|db| {
        let db = db.borrow();
        if let Some(receipt) = db.receipts.get(&receipt_id) {
            //call to other canisters to notify users
            let buyer_message = format!("Receipt for purchase of item ID {}: Amount transferred: {}", receipt.item_id, receipt.amount);
            let seller_message = format!("Receipt for sale of item ID {}: Amount recieved: {}", receipt.item_id, erceipt.amount);
            //inter-canister call notifications
            futures::join!(
                send_notification(receipt.buyer, buyer_message),
                send_notification(receipt.seller, seller_message)
            );
            Ok(())
        } else {
            Err("Receipt not found".to_string())
        }
    })
}

async fn send_notification(user: api::Principal, message: String) -> Result<(), String> {
    let notification_canister_id = api::id(); // verify actual id
    match call(notification_canister_id, "notify_user", (user, message)).await {
        Ok(()) => Ok(()),
        Err((_, mse)) => Err(format!("Failed to send notification: {}", msg)),
    }
}

#[query]
fn get_receipt(receipt_id: u64) -> Result<Receipt, String> {
    DB.with(|db| {
        db.borrow().receipts.get(&receipt_id)
            .cloned()
            .ok_or_else(|| "Receipt not found".to_string())
    })
}

#[query]
fn get_receipts_by_item(item_id: u64) -> Vec<Receipt> {
    DB.with(|db| {
        db.borrow().receipts.values()
            .filter(|r| r.item_id == item_id)
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
    let db: ReceiptDatabase = ic_cdk::storage::stable_restore().unwrap_or(ReceiptDatabase {
        receipts: HashMap::new(),
        next_receipt_id: 0,
    });
    DB.with(|inner_db| *inner_db.borrow_mut() = db);
}