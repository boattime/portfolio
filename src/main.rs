use log::{error, info};
use portfolio::{config::Config, error::Result, scheduler::Scheduler};
use std::process;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("Starting portfolio website generator...");

    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    let scheduler = Scheduler::new(config.clone());

    match scheduler.run().await {
        Ok(_) => {
            info!("Scheduler completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Scheduler failed: {}", e);
            Err(e)
        }
    }
}
