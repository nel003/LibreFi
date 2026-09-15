use crate::network::server::Server;
use crate::utils::db::{USERS_TABLE, User, get_db};
use crate::utils::get_mac_from_ip::get_mac_from_ip;
use redb::ReadableDatabase;

pub fn index(server: &mut Server) {
    server.get("/", |req, res| {
        if req.ip != "unknown" && !req.ip.is_empty() {
            if let Some(mac) = get_mac_from_ip(&req.ip) {
                let db = get_db();

                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(USERS_TABLE).unwrap();
                let existing = table.get(mac.as_str()).unwrap();

                match existing {
                    Some(record) => {
                        let mut user: User = serde_json::from_str(record.value()).unwrap();
                        if user.ip != req.ip {
                            user.ip = req.ip.clone();
                            drop(table);
                            drop(read_txn);

                            let write_txn = db.begin_write().unwrap();
                            {
                                let mut wtable = write_txn.open_table(USERS_TABLE).unwrap();
                                let json = serde_json::to_string(&user).unwrap();
                                wtable.insert(mac.as_str(), json.as_str()).unwrap();
                            }
                            write_txn.commit().unwrap();
                        }
                    }
                    None => {
                        drop(table);
                        drop(read_txn);

                        let new_user = User {
                            name: String::new(),
                            ip: req.ip.clone(),
                            paused: false,
                            pause_attempts: 0,
                            pause_day: 0,
                            paused_on: 0,
                            expires_on: 0,
                        };

                        let write_txn = db.begin_write().unwrap();
                        {
                            let mut wtable = write_txn.open_table(USERS_TABLE).unwrap();
                            let json = serde_json::to_string(&new_user).unwrap();
                            wtable.insert(mac.as_str(), json.as_str()).unwrap();
                        }
                        write_txn.commit().unwrap();

                        crate::debug_println!("-> New user registered: MAC={} IP={}", mac, req.ip);
                    }
                }
            } else {
                res.status = 403;
                res.body = b"<h1>This system is not for your device.</h1>".to_vec();
                res.content_type = String::from("text/html");
                return;
            }
        }

        match crate::network::server::Asset::get("index.html") {
            Some(file) => {
                res.status = 200;
                res.body = file.data.into_owned();
                res.content_type = String::from("text/html");
            }
            None => {
                res.status = 500;
                res.body = b"Could not load embedded index.html".to_vec();
            }
        }
    });
}
