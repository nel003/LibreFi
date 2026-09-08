use std::process::Command;
use crate::utils::cmds::has_command;
use crate::utils::db::{get_db, User, USERS_TABLE, CONFIG_TABLE};
use crate::routes::admin::QosPayload;
use redb::{ReadableDatabase, ReadableTable};
use std::time::SystemTime;

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
    let sysupgrade_script = r#"
        if ! grep -q '/opt/librefi/data.redb' /etc/sysupgrade.conf 2>/dev/null; then
            echo '/opt/librefi/data.redb' >> /etc/sysupgrade.conf
        fi
    "#;
    let _ = run_sh_cmd(sysupgrade_script, true);
    let lan_ip_script = format!(
        r#"
        current_ip=$(uci -q get network.lan.ipaddr)
        current_mask=$(uci -q get network.lan.netmask)
        
        if [ "$current_ip" != "{0}" ] || [ "$current_mask" != "255.255.255.0" ]; then
            uci set network.lan.ipaddr='{0}'
            uci set network.lan.netmask='255.255.255.0'
            uci commit network
            /etc/init.d/network reload
        fi
        "#,
        portal_ip
    );
    let _ = run_sh_cmd(&lan_ip_script, true);

    let firewall_script = r#"
        needs_reload=0

        has_rule=$(uci -q show firewall | grep -c "name='Allow-LibreFi-Admin-WAN'")
        if [ "$has_rule" -eq 0 ]; then
            uci add firewall rule
            uci set firewall.@rule[-1].name='Allow-LibreFi-Admin-WAN'
            uci set firewall.@rule[-1].src='wan'
            uci set firewall.@rule[-1].target='ACCEPT'
            uci set firewall.@rule[-1].proto='tcp'
            uci set firewall.@rule[-1].dest_port='80'
            uci commit firewall
            needs_reload=1
        fi

        has_ssh_drop=$(uci -q show firewall | grep -c "name='Drop-SSH-LAN'")
        if [ "$has_ssh_drop" -eq 0 ]; then
            uci add firewall rule
            uci set firewall.@rule[-1].name='Drop-SSH-LAN'
            uci set firewall.@rule[-1].src='lan'
            uci set firewall.@rule[-1].target='DROP'
            uci set firewall.@rule[-1].proto='tcp'
            uci set firewall.@rule[-1].dest_port='22'
            uci commit firewall
            needs_reload=1
        fi

        has_ssh_allow=$(uci -q show firewall | grep -c "name='Allow-SSH-WAN'")
        if [ "$has_ssh_allow" -eq 0 ]; then
            uci add firewall rule
            uci set firewall.@rule[-1].name='Allow-SSH-WAN'
            uci set firewall.@rule[-1].src='wan'
            uci set firewall.@rule[-1].target='ACCEPT'
            uci set firewall.@rule[-1].proto='tcp'
            uci set firewall.@rule[-1].dest_port='22'
            uci commit firewall
            needs_reload=1
        fi

        if [ "$needs_reload" -eq 1 ]; then
              /etc/init.d/firewall reload
        fi
    "#;
    let _ = run_sh_cmd(firewall_script, true);

    let dnsmasq_script = format!(
        r#"
        idx=0
        while uci -q get dhcp.@dnsmasq[$idx] >/dev/null; do
            port=$(uci -q get dhcp.@dnsmasq[$idx].port)
            if [ "$port" = "{0}" ]; then
                uci delete dhcp.@dnsmasq[$idx]
            else
                idx=$((idx+1))
            fi
        done

        uci -q delete dhcp.lan.dhcp_option
        uci add_list dhcp.lan.dhcp_option='6,8.8.8.8,{1}'
        uci -q delete dhcp.@dnsmasq[0].address

        # Google / Android
        uci add_list dhcp.@dnsmasq[0].address='/connectivitycheck.gstatic.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/connectivitycheck.android.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/clients1.google.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/clients3.google.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/clients4.google.com/{1}'

        # Apple / iOS / macOS
        uci add_list dhcp.@dnsmasq[0].address='/captive.apple.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.apple.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.appleiphonecell.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.airport.us/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.itools.info/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.ibook.info/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.thinkdifferent.us/{1}'

        # Microsoft / Windows
        uci add_list dhcp.@dnsmasq[0].address='/msftconnecttest.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.msftconnecttest.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/ipv6.msftconnecttest.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/www.msftncsi.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/ipv6.msftncsi.com/{1}'

        # Firefox / Mozilla
        uci add_list dhcp.@dnsmasq[0].address='/detectportal.firefox.com/{1}'

        # Other OEM (Xiaomi, Huawei, etc)
        uci add_list dhcp.@dnsmasq[0].address='/connect.rom.miui.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/connectivitycheck.platform.hicloud.com/{1}'

        uci commit dhcp
        /etc/init.d/dnsmasq restart
        "#,
        portal_dns_port, portal_ip
    );
    let _ = run_sh_cmd(&dnsmasq_script, true);

    let is_nft = has_command("nft");

    if is_nft {
        let commands = vec![
            ("nft add table inet librefi".to_string(), true),
            ("nft delete chain inet librefi captive_mangle".to_string(), true),
            ("nft delete chain inet librefi captive_nat".to_string(), true),
            ("nft delete chain inet librefi captive_forward".to_string(), true),
            ("nft delete chain inet librefi captive_mangle_post".to_string(), true),
            ("nft delete chain inet librefi captive_nat_post".to_string(), true),
            (r#"nft add set inet librefi allowed_macs { type ether_addr\; flags timeout\; }"#.to_string(), true),
            
            (r#"nft add chain inet librefi captive_mangle { type filter hook prerouting priority -150 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet librefi captive_mangle iifname "{}" ether saddr @allowed_macs return"#, lan_iface), false),
            (format!(r#"nft add rule inet librefi captive_mangle iifname "{}" meta mark set 99"#, lan_iface), false),
            
            (r#"nft add chain inet librefi captive_nat { type nat hook prerouting priority -100 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet librefi captive_nat iifname "{}" meta mark 99 tcp dport 80 dnat ip to "{}""#, lan_iface, portal_ip), false),
            
            (r#"nft add chain inet librefi captive_forward { type filter hook forward priority 0 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet librefi captive_forward iifname "{}" meta mark 99 drop"#, lan_iface), false),
            (format!(r#"nft add rule inet librefi captive_forward iifname "{}" ct state established,related accept"#, lan_iface), false),
            
            (r#"nft add chain inet librefi captive_mangle_post { type filter hook postrouting priority -150 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet librefi captive_mangle_post oifname "{}" ip ttl set 1"#, lan_iface), false),
            
            (r#"nft add chain inet librefi captive_nat_post { type nat hook postrouting priority 100 \; }"#.to_string(), false),
            (format!(r#"nft add rule inet librefi captive_nat_post oifname "{}" masquerade"#, wan_iface), false),
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
            (format!("iptables -t nat -A captive_nat -p tcp --dport 80 -j DNAT --to-destination \"{}\":80", portal_ip), false),

            ("iptables -t filter -N captive_forward".to_string(), false),
            (format!("iptables -t filter -A FORWARD -i \"{}\" -j captive_forward", lan_iface), false),
            ("iptables -t filter -A captive_forward -m mark --mark 99 -j DROP".to_string(), false),
            ("iptables -t filter -A captive_forward -m state --state ESTABLISHED,RELATED -j ACCEPT".to_string(), false),

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

pub fn allow_mac(mac: &str) {
    let db = crate::utils::db::get_db();
    let mut timeout = None;

    if let Ok(read_txn) = db.begin_read() {
        if let Ok(table) = read_txn.open_table(crate::utils::db::USERS_TABLE) {
            if let Ok(Some(value)) = table.get(mac) {
                if let Ok(user) = serde_json::from_str::<crate::utils::db::User>(value.value()) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u32;

                    if user.paused || user.expires_on <= now {
                        return; // Don't allow if paused or expired
                    }
                    timeout = Some(user.expires_on - now);
                }
            }
        }
    }

    let Some(t) = timeout else {
        return;
    };

    let is_nft = has_command("nft");
    let cmd = if is_nft {
        format!("nft add element inet librefi allowed_macs {{ {} timeout {}s }}", mac, t)
    } else {
        format!("ipset add allowed_macs {} timeout {}", mac, t)
    };
    let _ = run_sh_cmd(&cmd, true);
}

pub fn authorize_active_users() {
    let db = get_db();
    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(e) => {
            eprintln!("Failed to begin read transaction: {}", e);
            return;
        }
    };

    let table = match read_txn.open_table(USERS_TABLE) {
        Ok(t) => t,
        Err(_) => return, // Table might not exist yet
    };

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    for item in table.iter().unwrap() {
        if let Ok((key, value)) = item {
            let mac = key.value();
            if let Ok(user) = serde_json::from_str::<User>(value.value()) {
                if !user.paused && user.expires_on > now {
                    allow_mac(mac);
                    println!("Restored active session for MAC: {}", mac);
                }
            }
        }
    }
}

pub fn restore_qos_settings() {
    let db = get_db();
    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(_) => return,
    };

    let table = match read_txn.open_table(CONFIG_TABLE) {
        Ok(t) => t,
        Err(_) => return,
    };

    if let Ok(Some(value)) = table.get("qos") {
        if let Ok(payload) = serde_json::from_str::<QosPayload>(value.value()) {
            let download_kbps = (payload.download * 1000.0).round() as u32;
            let upload_kbps = (payload.upload * 1000.0).round() as u32;

            let script = format!(
                "if [ ! -f /etc/init.d/sqm ]; then\n\
                     if command -v apk >/dev/null 2>&1; then\n\
                         apk update && apk add sqm-scripts\n\
                     elif command -v opkg >/dev/null 2>&1; then\n\
                         opkg update && opkg install sqm-scripts\n\
                     else\n\
                         echo \"No supported package manager found\" >&2\n\
                         exit 1\n\
                     fi\n\
                 fi\n\
                 uci set sqm.openfi=queue\n\
                 uci set sqm.openfi.enabled='1'\n\
                 uci set sqm.openfi.interface='br-lan'\n\
                 uci set sqm.openfi.download='{}'\n\
                 uci set sqm.openfi.upload='{}'\n\
                 uci set sqm.openfi.qdisc='cake'\n\
                 uci set sqm.openfi.script='piece_of_cake.qos'\n\
                 uci commit sqm\n\
                 mkdir -p /var/lock\n\
                 /etc/init.d/sqm enable\n\
                 /etc/init.d/sqm restart",
                download_kbps, upload_kbps
            );

            match run_sh_cmd(&script, false) {
                Ok(_) => println!("-> QoS settings restored successfully: Download = {} Mbps, Upload = {} Mbps", payload.download, payload.upload),
                Err(e) => println!("-> Failed to restore QoS settings: {}", e),
            }
        } else {
            println!("-> Found 'qos' key but failed to parse JSON payload");
        }
    } else {
        println!("-> No saved QoS settings found in database. Skipping restoration.");
    }
}
