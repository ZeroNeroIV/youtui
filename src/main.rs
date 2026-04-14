use clap::Parser;
use youtui_rs::config::settings::Settings;
use youtui_rs::config::{is_development, log_to_terminal, set_debug_mode};
use youtui_rs::ui::app::App;
use youtui_rs::utils::logger::{init_logger, LoggerConfig};

#[derive(clap::Parser)]
struct Args {
    #[arg(long, default_value = "false")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    set_debug_mode(args.debug);

    let settings = Settings::load();
    let logger_config = LoggerConfig {
        is_dev: is_development(),
        to_terminal: log_to_terminal(),
        log_level: settings.log_level,
    };
    let _guard = init_logger(logger_config);

    let mut app = App::new().await?;
    app.run()?;
    Ok(())
}
