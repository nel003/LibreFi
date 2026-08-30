use std::collections::HashMap;
use std::io::Read;
use std::panic::{AssertUnwindSafe, catch_unwind};
use tiny_http::{Header, Response as TinyResponse, Server as TinyServer};

pub struct Request {
    pub url: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub ip: String,
}

pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_type: String,
}

type Handler = Box<dyn Fn(&Request, &mut Response)>;

pub struct Server {
    pub port: u16,
    listener: TinyServer,
    get_routes: HashMap<String, Handler>,
    post_routes: HashMap<String, Handler>,
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
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.get_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + 'static,
    {
        self.post_routes.insert(path.to_string(), Box::new(handler));
    }

    pub fn run(&self) {
        println!("Server running on port {}", self.port);

        for mut real_request in self.listener.incoming_requests() {
            let raw_url = real_request.url().to_string();

            let mut parts = raw_url.splitn(2, '?');
            let mut path = parts.next().unwrap_or("/").to_string();
            let query = parts.next().unwrap_or("").to_string();

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

            let mut body = Vec::new();
            let _ = real_request.as_reader().read_to_end(&mut body);

            let ip = headers
                .get("x-forwarded-for")
                .map(|h| h.to_string())
                .unwrap_or_else(|| {
                    real_request
                        .remote_addr()
                        .map(|addr| addr.ip().to_string())
                        .unwrap_or_else(|| String::from("unknown"))
                });

            let req = Request {
                url: path.clone(),
                query,
                headers,
                body,
                ip,
            };

            let mut res = Response {
                status: 404,
                body: b"404 - Your father Not Found".to_vec(),
                content_type: String::from("text/plain"),
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

                    let file_path = format!("dist/{}", safe_path);

                    if let Ok(contents) = std::fs::read(&file_path) {
                        res.status = 200;
                        res.body = contents;

                        if file_path.ends_with(".html") {
                            res.content_type = String::from("text/html");
                        } else if file_path.ends_with(".css") {
                            res.content_type = String::from("text/css");
                        } else if file_path.ends_with(".js") {
                            res.content_type = String::from("application/javascript");
                        } else if file_path.ends_with(".json") {
                            res.content_type = String::from("application/json");
                        } else if file_path.ends_with(".svg") {
                            res.content_type = String::from("image/svg+xml");
                        } else if file_path.ends_with(".woff2") {
                            res.content_type = String::from("font/woff2");
                        }
                    } else {
                        res.status = 404;
                        res.body = b"404 - Your father not found!".to_vec();
                        res.content_type = String::from("text/text");
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
            }

            let header =
                Header::from_bytes(&b"Content-Type"[..], res.content_type.as_bytes()).unwrap();

            let tiny_res = TinyResponse::from_data(res.body)
                .with_status_code(res.status)
                .with_header(header);

            let _ = real_request.respond(tiny_res);
        }
    }
}