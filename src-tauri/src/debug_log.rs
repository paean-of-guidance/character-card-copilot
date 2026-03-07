pub fn enabled() -> bool {
    if cfg!(debug_assertions) {
        return true;
    }

    matches!(
        std::env::var("CHARACTER_CARD_DEBUG"),
        Ok(value)
            if matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
    )
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::debug_log::enabled() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug_warn {
    ($($arg:tt)*) => {
        if $crate::debug_log::enabled() {
            eprintln!($($arg)*);
        }
    };
}
