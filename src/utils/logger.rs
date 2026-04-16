use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, EnvFilter};

pub struct LoggerConfig {
    pub is_dev: bool,
    pub to_terminal: bool,
    pub log_level: String,
}

static LOG_HANDLE: OnceLock<reload::Handle<EnvFilter, tracing_subscriber::Registry>> =
    OnceLock::new();

fn get_log_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("youtui-rs").join("logs"))
}

fn get_log_filename() -> String {
    let now = chrono::Local::now();
    format!("youtui-rs-{}.log", now.format("%Y-%m-%d_%H-%M-%S"))
}

pub fn init_logger(config: LoggerConfig) -> Option<non_blocking::WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if config.is_dev {
            EnvFilter::new("debug,youtui_rs=debug")
        } else {
            EnvFilter::new(&config.log_level)
        }
    });

    let (reload_layer, handle) = reload::Layer::new(filter);
    LOG_HANDLE.set(handle).expect("Logger handle already set");

    let registry = tracing_subscriber::registry().with(reload_layer);

    if let Some(log_dir) = get_log_dir() {
        let _ = std::fs::create_dir_all(&log_dir);
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, get_log_filename());
        let (non_blocking, guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_ansi(false)
            .with_writer(non_blocking);

        if config.to_terminal {
            let stderr_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_writer(std::io::stderr);

            registry.with(file_layer).with(stderr_layer).init();
        } else {
            registry.with(file_layer).init();
        }
        return Some(guard);
    }

    if config.to_terminal || config.is_dev {
        let stderr_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_writer(std::io::stderr);

        registry.with(stderr_layer).init();
    }
    None
}

pub fn update_log_level(level: &str) {
    if let Some(handle) = LOG_HANDLE.get() {
        let new_filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));
        handle
            .modify(|filter| *filter = new_filter)
            .expect("Failed to update log level");
    }
}
