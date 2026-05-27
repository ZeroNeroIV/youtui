pub mod settings;

use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_debug_mode(debug: bool) {
    DEBUG_MODE.store(debug, Ordering::SeqCst);
}

pub fn is_debug_enabled() -> bool {
    DEBUG_MODE.load(Ordering::SeqCst)
}

pub fn is_development() -> bool {
    cfg!(debug_assertions) || is_debug_enabled()
}

pub fn log_to_terminal() -> bool {
    is_debug_enabled()
}
