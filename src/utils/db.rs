use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// 1. Define your struct
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub ip: String,
    pub paused: bool,
    pub pause_attempts: u8,  // resets to 0 each new day
    pub pause_day: u32,      // day number (now / 86400) of the last pause reset
    pub paused_on: u32,
    pub expires_on: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Rate {
    pub price: f64,
    pub time: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Voucher {
    pub code: String,
    pub price: f64,
    pub time: u32,
    pub used: bool,
}


pub static DB: OnceLock<Database> = OnceLock::new();

pub const USERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("users");
pub const RATES_TABLE: TableDefinition<u32, &str> = TableDefinition::new("rates");
pub const VOUCHERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("vouchers");

pub fn init_db(db_path: &str) {
    let db = Database::create(db_path).expect("Failed to create database");

    // Initialize tables
    let write_txn = db.begin_write().unwrap();
    {
        let _ = write_txn.open_table(USERS_TABLE).unwrap();
        let _ = write_txn.open_table(RATES_TABLE).unwrap();
        let _ = write_txn.open_table(VOUCHERS_TABLE).unwrap();
    }
    write_txn.commit().unwrap();

    // Store the database in the global static variable
    DB.set(db).expect("Database was already initialized");
}

pub fn get_db() -> &'static Database {
    DB.get().expect("Database has not been initialized yet!")
}
