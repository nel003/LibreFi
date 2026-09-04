use std::process::Command;
use crate::utils::cmds::has_command;

pub fn run_sh_cmd(cmd: &str, ignore_errors: bool) -> Result<(), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("Failed to execute command '{}': {}", cmd, e))?;

    if !output.status.success() && !ignore_errors {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command '{}' failed with status {}:\n{}", cmd, output.status, err));
    }

    if output.status.success() {
        println!("Successfully executed: {}", cmd);
    }

    Ok(())
}

pub fn setup_captive_portal(
    lan_iface: &str,
    wan_iface: &str,
    portal_ip: &str,
    portal_dns_port: &str,
) {
    let is_nft = has_command("nft");

    if is_nft {
        let commands = vec![
            ("nft add table inet fw4".to_string(), true),
            ("nft delete chain inet fw4 captive_mangle".to_string(), true),
            ("nft delete chain inet fw4 captive_nat".to_string(), true),
            ("nft delete chain inet fw4 captive_forward".to_string(), true),
            ("nft delete chain inet fw4 captive_mangle_post".to_string(), true),
            ("nft delete chain inet fw4 captive_nat_post".to_string(), true),
            (r#"nft add set inet fw4 allowed_macs { type ether_addr\; flags timeout\; }"#.to_string(), true),
            
            (r#"nft add chain inet fw4 captive_mangle { type filter hook prerouting priority -150 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet fw4 captive_mangle iifname "{}" ether saddr @allowed_macs return"#, lan_iface), false),
            (format!(r#"nft add rule inet fw4 captive_mangle iifname "{}" meta mark set 99"#, lan_iface), false),
            
            (r#"nft add chain inet fw4 captive_nat { type nat hook prerouting priority -100 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet fw4 captive_nat iifname "{}" meta mark 99 udp dport 53 dnat ip to "{}:{}""#, lan_iface, portal_ip, portal_dns_port), false),
            (format!(r#"nft add rule inet fw4 captive_nat iifname "{}" meta mark 99 tcp dport 53 dnat ip to "{}:{}""#, lan_iface, portal_ip, portal_dns_port), false),
            (format!(r#"nft add rule inet fw4 captive_nat iifname "{}" meta mark 99 tcp dport 80 dnat ip to "{}""#, lan_iface, portal_ip), false),
            
            (r#"nft add chain inet fw4 captive_forward { type filter hook forward priority 0 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet fw4 captive_forward iifname "{}" ct state established,related accept"#, lan_iface), false),
            (format!(r#"nft add rule inet fw4 captive_forward iifname "{}" meta mark 99 drop"#, lan_iface), false),
            
            (r#"nft add chain inet fw4 captive_mangle_post { type filter hook postrouting priority -150 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet fw4 captive_mangle_post oifname "{}" ip ttl set 1"#, lan_iface), false),
            
            (r#"nft add chain inet fw4 captive_nat_post { type nat hook postrouting priority 100 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet fw4 captive_nat_post oifname "{}" masquerade"#, wan_iface), false),
        ];

        for (cmd, ignore_err) in commands {
            if let Err(e) = run_sh_cmd(&cmd, ignore_err) {
                eprintln!("Captive portal setup error: {}", e);
                return;
            }
        }
    } else {
        // Flush chains
        let chains = ["captive_mangle", "captive_nat", "captive_forward", "captive_mangle_post", "captive_nat_post"];
        for chain in chains.iter() {
            let _ = run_sh_cmd(&format!("iptables -t mangle -D PREROUTING -j {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t nat -D PREROUTING -j {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t filter -D FORWARD -j {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t mangle -D POSTROUTING -j {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t nat -D POSTROUTING -j {}", chain), true);
            
            let _ = run_sh_cmd(&format!("iptables -t mangle -F {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t mangle -X {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t nat -F {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t nat -X {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t filter -F {}", chain), true);
            let _ = run_sh_cmd(&format!("iptables -t filter -X {}", chain), true);
        }

        let commands = vec![
            ("ipset create allowed_macs hash:mac timeout 2147483 -exist".to_string(), false),

            ("iptables -t mangle -N captive_mangle".to_string(), false),
            (format!("iptables -t mangle -A PREROUTING -i \"{}\" -j captive_mangle", lan_iface), false),
            ("iptables -t mangle -A captive_mangle -m set --match-set allowed_macs src -j RETURN".to_string(), false),
            ("iptables -t mangle -A captive_mangle -j MARK --set-mark 99".to_string(), false),

            ("iptables -t nat -N captive_nat".to_string(), false),
            (format!("iptables -t nat -A PREROUTING -i \"{}\" -m mark --mark 99 -j captive_nat", lan_iface), false),
            (format!("iptables -t nat -A captive_nat -p udp --dport 53 -j DNAT --to-destination \"{}:{}\"", portal_ip, portal_dns_port), false),
            (format!("iptables -t nat -A captive_nat -p tcp --dport 53 -j DNAT --to-destination \"{}:{}\"", portal_ip, portal_dns_port), false),
            (format!("iptables -t nat -A captive_nat -p tcp --dport 80 -j DNAT --to-destination \"{}\":80", portal_ip), false),

            ("iptables -t filter -N captive_forward".to_string(), false),
            (format!("iptables -t filter -A FORWARD -i \"{}\" -j captive_forward", lan_iface), false),
            ("iptables -t filter -A captive_forward -m state --state ESTABLISHED,RELATED -j ACCEPT".to_string(), false),
            ("iptables -t filter -A captive_forward -m mark --mark 99 -j DROP".to_string(), false),

            ("iptables -t mangle -N captive_mangle_post".to_string(), false),
            (format!("iptables -t mangle -A POSTROUTING -o \"{}\" -j captive_mangle_post", lan_iface), false),
            ("iptables -t mangle -A captive_mangle_post -j TTL --ttl-set 1".to_string(), false),

            ("iptables -t nat -N captive_nat_post".to_string(), false),
            (format!("iptables -t nat -A POSTROUTING -o \"{}\" -j captive_nat_post", wan_iface), false),
            ("iptables -t nat -A captive_nat_post -j MASQUERADE".to_string(), false),
        ];

        for (cmd, ignore_err) in commands {
            if let Err(e) = run_sh_cmd(&cmd, ignore_err) {
                eprintln!("Captive portal setup error: {}", e);
                return;
            }
        }
    }
}
