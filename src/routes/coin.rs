use crate::network::server::Server;

pub fn coin(server: &mut Server) {
    server.get("/coin", |_, res| {
        res.status = 418;
        res.body = serde_json::json!({
            "error": "Service not implemented"
        }).to_string().into_bytes();
        res.content_type = String::from("application/json");
    });

    server.post("/coin", |_, res| {
        res.status = 418;
        res.body = serde_json::json!({
            "error": "Service not implemented"
        }).to_string().into_bytes();
        res.content_type = String::from("application/json");
    });
}