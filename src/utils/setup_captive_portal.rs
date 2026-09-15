use std::process::Command;
use crate::utils::cmds::has_command;
use crate::utils::db::{get_db, User, USERS_TABLE, CONFIG_TABLE};
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
        crate::debug_println!("Successfully executed: {}", cmd);
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
        uci add_list dhcp.@dnsmasq[0].address='/play.googleapis.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/developers.google.cn/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/g.cn/{1}'

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

        # Other OEM (Xiaomi, Huawei, Oppo, Vivo, Samsung)
        uci add_list dhcp.@dnsmasq[0].address='/connect.rom.miui.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/connectivitycheck.platform.hicloud.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/connectivitycheck.oppomobile.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/wifi.vivo.com.cn/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/conn1.samsung.com/{1}'
        uci add_list dhcp.@dnsmasq[0].address='/conn2.samsung.com/{1}'

        uci commit dhcp
        /etc/init.d/dnsmasq restart
        "#,
        portal_dns_port, portal_ip
    );
    let _ = run_sh_cmd(&dnsmasq_script, true);

    let is_nft = has_command("nft");

    if !is_nft {
        eprintln!("nftables is not supported on this router. Captive portal will not be configured.");
        return;
    }

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
            crate::debug_eprintln!("Captive portal setup error: {}", e);
            return;
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
                        return; 
                    }
                    timeout = Some(user.expires_on - now);
                }
            }
        }
    }

    let Some(t) = timeout else {
        return;
    };

    let cmd = format!("nft add element inet librefi allowed_macs {{ {} timeout {}s }}", mac, t);
    let _ = run_sh_cmd(&cmd, true);
}

pub fn authorize_active_users() {
    let db = get_db();
    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(e) => {
            crate::debug_eprintln!("Failed to begin read transaction: {}", e);
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
                    crate::debug_println!("Restored active session for MAC: {}", mac);
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
        if let Ok(payload) = serde_json::from_str::<crate::routes::admin::qos::QosPayload>(value.value()) {
            if let Err(e) = crate::routes::admin::qos::apply_qos(&payload) {
                crate::debug_eprintln!("Failed to restore QoS on boot: {}", e);
            } else {
                crate::debug_println!("QoS settings restored successfully.");
            }
        } else {
            crate::debug_println!("-> Found 'qos' key but failed to parse JSON payload");
        }
    } else {
        crate::debug_println!("-> No saved QoS settings found in database. Skipping restoration.");
    }
}
