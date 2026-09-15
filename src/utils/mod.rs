pub mod get_mac_from_ip;
pub mod db;
pub mod crypto;
pub mod cmds;
pub mod setup_captive_portal;

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        if std::env::var("LIBREFI_DEBUG").as_deref() == Ok("true") {
            std::println!($($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! debug_eprintln {
    ($($arg:tt)*) => {{
        if std::env::var("LIBREFI_DEBUG").as_deref() == Ok("true") {
            std::eprintln!($($arg)*);
        }
    }};
}