use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// 1. Define your struct
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub ip: String,
    pub paused: bool,
    pub pause_attempts: u8, // resets to 0 each new day
    pub pause_day: u32,     // day number (now / 86400) of the last pause reset
    pub paused_on: u32,
    pub expires_on: u32,
}

pub static DB: OnceLock<Database> = OnceLock::new();

pub const USERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("users");
pub const RATES_TABLE: TableDefinition<u32, &str> = TableDefinition::new("rates");
pub const VOUCHERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("vouchers");
pub const CONFIG_TABLE: TableDefinition<&str, &str> = TableDefinition::new("config");
pub const SALES_TABLE: TableDefinition<u32, u32> = TableDefinition::new("sales");

pub fn init_db(db_path: &str) {
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let db = Database::create(db_path).expect("Failed to create database");

    // Initialize tables
    let write_txn = db.begin_write().unwrap();
    {
        let _ = write_txn.open_table(USERS_TABLE).unwrap();
        let _ = write_txn.open_table(RATES_TABLE).unwrap();
        let _ = write_txn.open_table(VOUCHERS_TABLE).unwrap();
        let _ = write_txn.open_table(CONFIG_TABLE).unwrap();
        let _ = write_txn.open_table(SALES_TABLE).unwrap();
    }
    write_txn.commit().unwrap();

    // Store the database in the global static variable
    DB.set(db).expect("Database was already initialized");
}

pub fn get_db() -> &'static Database {
    DB.get().expect("Database has not been initialized yet!")
}

/// Convert inserted amount to time in seconds using a greedy algorithm based on RATES_TABLE.
pub fn convert_amount_to_time(amount: u32) -> u32 {
    let db = get_db();
    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(_) => return 0,
    };

    let table = match read_txn.open_table(RATES_TABLE) {
        Ok(t) => t,
        Err(_) => return 0,
    };

    let mut rates = Vec::new();
    for item in table.iter().unwrap() {
        if let Ok((_key, value)) = item {
            if let Ok(rate) = serde_json::from_str::<Rate>(value.value()) {
                rates.push(rate);
            }
        }
    }

    // Sort descending by price for greedy algorithm
    rates.sort_by(|a, b| {
        b.price
            .partial_cmp(&a.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut remaining = amount as f64;
    let mut total_seconds = 0;

    for rate in rates {
        if rate.price <= 0.0 {
            continue;
        }
        let count = (remaining / rate.price).floor() as u32;
        if count > 0 {
            total_seconds += count * rate.time;
            remaining -= (count as f64) * rate.price;
        }
    }

    total_seconds
}

/// Add time (in seconds) to a user, returning the updated User.
pub fn add_time_to_user(mac: &str, seconds: u32) -> Result<User, String> {
    let db = get_db();
    let write_txn = db.begin_write().map_err(|e| e.to_string())?;

    let mut user = {
        let table = write_txn
            .open_table(USERS_TABLE)
            .map_err(|e| e.to_string())?;
        match table.get(mac) {
            Ok(Some(value)) => {
                serde_json::from_str::<User>(value.value()).map_err(|e| e.to_string())?
            }
            _ => return Err("User not found".to_string()),
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    if user.expires_on < now {
        user.expires_on = now;
    }
    user.expires_on += seconds;

    {
        let mut table = write_txn
            .open_table(USERS_TABLE)
            .map_err(|e| e.to_string())?;
        table
            .insert(mac, serde_json::to_string(&user).unwrap().as_str())
            .map_err(|e| e.to_string())?;
    }
    write_txn.commit().map_err(|e| e.to_string())?;

    Ok(user)
}

// pub fn remove_user(mac: &str) -> Result<(), redb::Error> {
//     let db = get_db();
//     let write_txn = db.begin_write()?;
//     {
//         let mut table = write_txn.open_table(USERS_TABLE)?;
//         table.remove(mac)?;
//     }
//     write_txn.commit()?;
//     Ok(())
// }

pub fn record_sale(amount: u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Default to GMT+8 (Philippines) for daily calculation since coinslot uses Pesos
    let local_time = now + 8 * 3600;
    let day = (local_time / 86400) as u32;

    let db = get_db();
    if let Ok(write_txn) = db.begin_write() {
        if let Ok(mut table) = write_txn.open_table(SALES_TABLE) {
            let current_total = match table.get(&day) {
                Ok(Some(v)) => v.value(),
                _ => 0,
            };
            let _ = table.insert(&day, current_total + amount);
        }
        let _ = write_txn.commit();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Rate {
    pub price: f64,
    pub time: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Voucher {
    pub code: String,
    pub price: f64,
    pub time: u32,
    pub used: bool,
    #[serde(default)]
    pub created_at: Option<u64>,
}
