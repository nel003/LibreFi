
use std::process::Command;
use crate::network::server::Server;

pub fn status(server: &mut Server) {
    server.get("/status", |_req, res| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(r#"awk '/MemTotal/ {t=$2} /MemAvailable/ {a=$2} END {printf "{\n  \"ram_used_pct\": %.1f,\n  \"ram_free_pct\": %.1f,\n", ((t-a)/t)*100, (a/t)*100}' /proc/meminfo && df /overlay | awk 'NR==2 {gsub("%","",$5); printf "  \"storage_used_pct\": %d,\n  \"storage_free_pct\": %d\n}\n", $5, 100-$5}'"#)
            .output()
            .expect("Failed to execute shell command");

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("Command failed:\n{}", error);
            
            // Set a 500 error instead of silently returning the default 404
            res.status = 500;
            res.body = format!("Command failed: {}", error).into_bytes();
            return; 
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        res.status = 200;
        res.body = stdout.into_owned().into_bytes();
    });
}