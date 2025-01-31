use candid::{CandidType, Deserialize};
use ic_cdk::{api, update, query};
use ic_cdk_macros::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct TokenBalance {
    principal: api::Principal,
    balance: u64, //maybe needs to be float or use special intToken\coin
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Transaction {
    id: u64,
    buyer: api::Principal,
    seller: api:Principal,
    amount: u64,
    post_id: u64,
}

#[derive(CandidType, Deserialize, Serialize)]
struct TokenDatabase {
    balances: HashMap<api::Principal, TokenBalance>,
    transactions: HashMap<u64, Transaction>,
    next_transaction_id: u64,
}

thread_local! {
    static DB:
    std::cell::RefCell<TokenDatabase> = std::cell::RefCell::new(TokenDatabase {
        balances: HashMap::new(),
        transactions: HashMap::new(),
        next_transaction_id: 0,
    });
}

#[init]
fn init(){
    //likely unneeded to initialize database with token distribution
}

#[update]
fn mint_tokens(principal: api::Principal, amount: u64) -> Result<(), String> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let balance = db.balances.entry(principal).or_insert(TokenBalance {
            principal,
            balance: 0,
        });
        balance.balance = balance.balance.checked_add(amount).ok_or("Overflow")?;
        Ok(())
    })
}

#[update]
fn transfer_tokens(to: api::Principal, amount: u64) -> Result<(), String> {
    let caller = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let from_balance = db.balance.get_mut(&caller).ok_or("Sender has no balance")?;

        from_balance.balance = from_balance.balance.checked_sub(amount).ok_or("Insufficient balance")?;
        let to_balance = db.balances.entry(to).or_insert(TokenBalance {
            principal: to,
            balance: 0,
        });
        to_balance.balance = to_balance.balance.checked_add(amount).ok_or("Overflow")?;

        let transaction_id = db.next_transaction_id;
        db.transactions.insert(transaction_id, Transaction {
            id: transaction_id,
            buyer: caller,
            seller: to,
            amount,
            post_id: 0, //set when buying an item?
        });
        db.next_transaction_id += 1;
        Ok(())
    })
}

#[update]
fn buy_item(post_id: u64, seller: api::Principal, price: u64) -> Result<u64, String> {
    let buyer = api::caller();
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let buyer_balance = db.balances.get_mut(&buyer).ok_or("Buyer has no balance")?;
        let seller_balance = db.balances.entry(seller).or_insert(TokenBalance {
            principal: seller,
            balance: 0,
        });
        buyer_balance.balance = buyer_balance.balance.checked_sub(price).ok_or("Insufficient funds")?;
        seller_balance.balance = seller_balance.balance.checked_add(price).ok_or("Overflow")?;
        let transaction_id = db.next_transaction_id;
        db.transactions.insert(transaction_id, Transaction {
            id: transaction_id,
            buyer,
            seller,
            amount: price,
            post_id,
        });
        db.next_transaction_id += 1;
        Ok(transaction_id)
    })
}

#[query]
fn get_balance(principal: api::Principal) -> Result<u64, String> {
    DB.with(|db| {
        db.borrow().balances.get(&principal)
            .map(|b| b.balance)
            .ok_or_else(|| "User not found".to_string())
    })
}

#[query]
fn get_transaction(transaction_id: u64) -> Result<Transaction, String> (
    DB.with(|db| {
        db.borrow().transactions,get(&transaction_id)
            .cloned()
            .ok_or_else(|| "Transaction not found".to_string())
    })
)

#[pre_upgrade]
fn pre_upgrade() {
    DB.with(|db| ic_cdk::storage::stable_save((db.borrow().clone(),)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let db: TokenDatabase = ic_cdk::storage::stable_restore().unwrap_or(TokenDatabase {
        balances: HashMap::new(),
        transactions: HashMap::new(),
        next_transaction_id: 0,
    });
    DB.with(|inner_db| *inner_db.borrow_mut() = db);
}