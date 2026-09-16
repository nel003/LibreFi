use crate::network::server::Server;
use crate::utils::db::{RATES_TABLE, get_db};
use redb::{ReadableDatabase, ReadableTable};

pub fn rates(server: &mut Server) {
    server.get("/api/rates", |_req, res| {
        let db = get_db();
        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(RATES_TABLE).unwrap();

        let mut rows: Vec<serde_json::Value> = Vec::new();

        for item in table.iter().unwrap() {
            let (key, value) = item.unwrap();
            let id = key.value();
            let rate: serde_json::Value =
                serde_json::from_str(value.value()).unwrap_or(serde_json::Value::Null);

            rows.push(serde_json::json!({ "id": id, "rate": rate }));
        }

        res.status = 200;
        res.body = serde_json::json!({ "count": rows.len(), "rows": rows })
            .to_string()
            .into_bytes();
        res.content_type = String::from("application/json");
    });
}
