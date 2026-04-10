use std::path::PathBuf;
use tracing_appender::non_blocking;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::{is_development, log_to_file, log_to_terminal};

fn get_log_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("youtui-rs").join("logs"))
}

pub fn init_logger() {
    let is_dev = is_development();
    let to_file = log_to_file();
    let to_terminal = log_to_terminal();

    if !is_dev && to_file {
        return;
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_dev {
            EnvFilter::new("debug,youtui_rs=debug")
        } else {
            EnvFilter::new("info")
        }
    });

    let registry = tracing_subscriber::registry().with(filter);

    if is_dev && to_file {
        if let Some(log_dir) = get_log_dir() {
            let _ = std::fs::create_dir_all(&log_dir);
            let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "youtui.log");
            let (non_blocking, _guard) = non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_ansi(false)
                .with_writer(non_blocking);

            if to_terminal {
                let stdout_layer = fmt::layer()
                    .with_target(true)
                    .with_level(true)
                    .with_writer(std::io::stdout);

                registry.with(file_layer).with(stdout_layer).init();
            } else {
                registry.with(file_layer).init();
            }
            return;
        }
    }

    if is_dev && to_terminal {
        let stdout_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_writer(std::io::stdout);

        registry.with(stdout_layer).init();
        return;
    }

    if is_dev {
        let stdout_layer = fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_writer(std::io::stdout);

        registry.with(stdout_layer).init();
    }
}
