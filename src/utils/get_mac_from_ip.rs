use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn get_mac_from_ip(target_ip: &str) -> Option<String> {
    let path = Path::new("/proc/net/arp");
    let file = File::open(&path).ok()?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1).flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() >= 4 {
            let ip = parts[0];
            let mac = parts[3];

            if ip == target_ip {
                if mac != "00:00:00:00:00:00" {
                    return Some(mac.to_string());
                }
            }
        }
    }
    
    None 
}