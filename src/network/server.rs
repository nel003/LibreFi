use redb::ReadableDatabase;
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use tiny_http::{Header, ReadWrite, Response as TinyResponse, Server as TinyServer};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist"]
pub struct Asset;

pub struct Request {
    pub body: Vec<u8>,
    pub ip: String,
}

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_type: String,
    pub headers: HashMap<String, String>,
}

type Handler = Box<dyn Fn(&Request, &mut Response)>;
pub type WsHandler =
    Arc<dyn Fn(tungstenite::WebSocket<Box<dyn ReadWrite + Send>>, String) + Send + Sync>;

pub struct Server {
    pub port: u16,
    listener: TinyServer,
    get_routes: HashMap<String, Handler>,
    post_routes: HashMap<String, Handler>,
    put_routes: HashMap<String, Handler>,
    delete_routes: HashMap<String, Handler>,
    ws_routes: HashMap<String, WsHandler>,
}

impl Server {
    pub fn new(port: u16) -> Self {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TinyServer::http(&addr).unwrap();

        Server {
            port,
            listener,
            get_routes: HashMap::new(),
            post_routes: HashMap::new(),
            put_routes: HashMap::new(),
            delete_routes: HashMap::new(),
            ws_routes: HashMap::new(),
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.get_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn ws<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(tungstenite::WebSocket<Box<dyn ReadWrite + Send>>, String) + Send + Sync + 'static,
    {
        self.ws_routes.insert(path.to_string(), Arc::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.post_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.put_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.delete_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn run(&self) {
        println!("Server running on port {}", self.port);

        for mut real_request in self.listener.incoming_requests() {
            let raw_url = real_request.url().to_string();

            let mut parts = raw_url.splitn(2, '?');
            let mut path = parts.next().unwrap_or("/").to_string();
            let _query = parts.next().unwrap_or("").to_string();

            if path.len() > 1 && path.ends_with('/') {
                path.pop();
            }

            let method = format!("{:?}", real_request.method()).to_lowercase();

            println!(
                "-> Received {} request for: '{}' (mapped to '{}')",
                method.to_uppercase(),
                raw_url,
                path
            );

            let mut headers = HashMap::new();
            for header in real_request.headers() {
                headers.insert(
                    header.field.as_str().to_string().to_lowercase(),
                    header.value.as_str().to_string(),
                );
            }

            let ip = headers
                .get("x-forwarded-for")
                .map(|h| h.to_string())
                .unwrap_or_else(|| {
                    real_request
                        .remote_addr()
                        .map(|addr| addr.ip().to_string())
                        .unwrap_or_else(|| String::from("unknown"))
                });

            let is_websocket_upgrade = headers
                .get("upgrade")
                .map(|v| v.to_lowercase() == "websocket")
                .unwrap_or(false);

            if method == "get" && is_websocket_upgrade {
                if let Some(handler) = self.ws_routes.get(&path) {
                    if let Some(key) = headers.get("sec-websocket-key") {
                        let accept_key = tungstenite::handshake::derive_accept_key(key.as_bytes());
                        let response = TinyResponse::empty(101)
                            .with_header(
                                Header::from_bytes(&b"Upgrade"[..], &b"websocket"[..]).unwrap(),
                            )
                            .with_header(
                                Header::from_bytes(&b"Connection"[..], &b"Upgrade"[..]).unwrap(),
                            )
                            .with_header(
                                Header::from_bytes(
                                    &b"Sec-WebSocket-Accept"[..],
                                    accept_key.as_bytes(),
                                )
                                .unwrap(),
                            );

                        let stream = real_request.upgrade("websocket", response);
                        let ws = tungstenite::WebSocket::from_raw_socket(
                            stream,
                            tungstenite::protocol::Role::Server,
                            None,
                        );
                        let client_ip = ip.clone();
                        let handler = Arc::clone(handler);
                        std::thread::spawn(move || {
                            handler(ws, client_ip);
                        });
                        continue;
                    }
                }
            }

            let mut body = Vec::new();
            let _ = real_request.as_reader().read_to_end(&mut body);

            let req = Request { body, ip };

            let mut res = Response {
                status: 404,
                body: b"404 - Your father Not Found".to_vec(),
                content_type: String::from("text/plain"),
                headers: HashMap::new(),
            };

            if method == "get" {
                if let Some(handler) = self.get_routes.get(&path) {
                    if catch_unwind(AssertUnwindSafe(|| handler(&req, &mut res))).is_err() {
                        res.status = 500;
                        res.body = b"500 - Internal Server Error".to_vec();
                    }
                } else {
                    let mut safe_path = path.trim_start_matches('/').replace("..", "");

                    if safe_path.is_empty() {
                        safe_path = String::from("index.html");
                    }

                    let mut contents_res = Asset::get(&safe_path).map(|f| f.data.into_owned()).ok_or(());

                    let mut is_cpd = false;
                    for cp_path in [
                        "/hotspot-detect.html",
                        "/generate_204",
                        "/gen_204",
                        "/ncsi.txt",
                        "/connecttest.txt",
                        "/success.txt",
                        "/canonical.html",
                        "/library/test/success.html",
                    ] {
                        if path.contains(cp_path) {
                            is_cpd = true;
                            break;
                        }
                    }

                    let mut has_access = false;
                    if let Some(mac) = crate::utils::get_mac_from_ip::get_mac_from_ip(&req.ip) {
                        let db = crate::utils::db::get_db();
                        if let Ok(read_txn) = db.begin_read() {
                            if let Ok(table) = read_txn.open_table(crate::utils::db::USERS_TABLE) {
                                if let Ok(Some(record)) = table.get(mac.as_str()) {
                                    if let Ok(user) = serde_json::from_str::<crate::utils::db::User>(
                                        record.value(),
                                    ) {
                                        let now = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                            as u32;
                                        if user.expires_on > now && !user.paused {
                                            has_access = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut handled_cpd = false;
                    if is_cpd && has_access {
                        handled_cpd = true;
                        if path.contains("hotspot-detect.html") || path.contains("success.html") {
                            res.status = 200;
                            res.body = b"<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>".to_vec();
                            res.content_type = String::from("text/html");
                        } else if path.contains("success.txt") {
                            res.status = 200;
                            res.body = b"success\n".to_vec();
                            res.content_type = String::from("text/plain");
                        } else {
                            res.status = 204;
                            res.body = Vec::new();
                        }
                    } else if is_cpd && !has_access {
                        handled_cpd = true;
                        res.status = 302;
                        res.headers
                            .insert("Location".to_string(), "http://10.0.0.1/".to_string());
                        res.body = b"Found".to_vec();
                    } else if contents_res.is_err() {
                        safe_path = String::from("index.html");
                        contents_res = Asset::get(&safe_path).map(|f| f.data.into_owned()).ok_or(());
                    }

                    if !handled_cpd {
                        if let Ok(contents) = contents_res {
                            res.status = 200;
                            res.body = contents;

                            if safe_path.ends_with(".html") {
                                res.content_type = String::from("text/html");
                            } else if safe_path.ends_with(".css") {
                                res.content_type = String::from("text/css");
                            } else if safe_path.ends_with(".js") {
                                res.content_type = String::from("application/javascript");
                            } else if safe_path.ends_with(".json") {
                                res.content_type = String::from("application/json");
                            } else if safe_path.ends_with(".svg") {
                                res.content_type = String::from("image/svg+xml");
                            } else if safe_path.ends_with(".woff2") {
                                res.content_type = String::from("font/woff2");
                            }
                        } else {
                            res.status = 404;
                            res.body = b"404 - index.html not found!".to_vec();
                            res.content_type = String::from("text/plain");
                        }
                    }
                }
            } else if method == "post" {
                if let Some(handler) = self.post_routes.get(&path) {
                    if catch_unwind(AssertUnwindSafe(|| handler(&req, &mut res))).is_err() {
                        res.status = 500;
                        res.body = b"500 - Internal Server Error".to_vec();
                    }
                } else {
                    println!("   [!] No POST route found for '{}'", path);
                }
            } else if method == "put" {
                if let Some(handler) = self.put_routes.get(&path) {
                    if catch_unwind(AssertUnwindSafe(|| handler(&req, &mut res))).is_err() {
                        res.status = 500;
                        res.body = b"500 - Internal Server Error".to_vec();
                    }
                } else {
                    println!("   [!] No PUT route found for '{}'", path);
                }
            } else if method == "delete" {
                if let Some(handler) = self.delete_routes.get(&path) {
                    if catch_unwind(AssertUnwindSafe(|| handler(&req, &mut res))).is_err() {
                        res.status = 500;
                        res.body = b"500 - Internal Server Error".to_vec();
                    }
                } else {
                    println!("   [!] No DELETE route found for '{}'", path);
                }
            } else if method == "options" {
                res.status = 204;
                res.body = Vec::new();
            }

            let content_type_header =
                Header::from_bytes(&b"Content-Type"[..], res.content_type.as_bytes()).unwrap();
            let cors_origin =
                Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();
            let cors_methods = Header::from_bytes(
                &b"Access-Control-Allow-Methods"[..],
                &b"GET, POST, PUT, DELETE, OPTIONS"[..],
            )
            .unwrap();
            let cors_headers = Header::from_bytes(
                &b"Access-Control-Allow-Headers"[..],
                &b"Content-Type, Authorization"[..],
            )
            .unwrap();

            let mut tiny_res = TinyResponse::from_data(res.body)
                .with_status_code(res.status)
                .with_header(content_type_header)
                .with_header(cors_origin)
                .with_header(cors_methods)
                .with_header(cors_headers);

            for (k, v) in res.headers {
                if let Ok(header) = Header::from_bytes(k.as_bytes(), v.as_bytes()) {
                    tiny_res = tiny_res.with_header(header);
                }
            }

            let _ = real_request.respond(tiny_res);
        }
    }
}
