use candid::{CandidType, Deserialize};
use ic_cdk::{api, update, query};
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Notification {
    id: u64,
    user: api::Principal,
    message: String, //add more structured data
    timestamp: u64,
}

struct NotificationDatabase {
    notifications: HashMap<api::Principal, Vec<Notification>>, next_notification_id: u64,
}

thread_local! {
    static DB:
    std::cell::RefCell<NotificationDatabase> = std::cell::RefCell::new(NotificationDatabase {
        notifications: HashMap::new(),
        next_notification_id: 0,
    });
}

#[init]
fn init() {
    //init databse?
}

#[update]
fn notify_user(user: api::Principal, message: String) -> u64 {
    let timestamp = api::time() / 1_000_000;
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let notification_id = db.next_notification_id;
        db.notification.entry(user).or_default().push(Notification {
            id: notification_id,
            user,
            message,
            timestamp,
        });
        db.next_notification_id += 1;
        notification_id
    })
}

#[query]
fn get_notifications(user: api::Principal) -> Vec<Notification> {
    DB.with(|db| {
        db.borrow().notification.get(&user).cloud().unwrap_or_default()
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    DB.with(|db| ic_cdk::storage::stable_save((db.borrow().clone(),)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let db: NotificationDatabase = ic_cdk::storage::stable_restore().unwrap_or(NotificationDatabase {
        notification: HashMap::new(),
        next_notification_id: 0,
    });
    DB.with(|inner_db| *inner_db.borrow_mut() = db);
}