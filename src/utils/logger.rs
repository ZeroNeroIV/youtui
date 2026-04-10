use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub fn init_logger() {
    let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).init();
}
