use log::{error, info};
use portfolio::{
    config::Config,
    error::Result,
    models::{LogEntry, LogLevel, Metric, Trace},
    scheduler::Scheduler,
    storage::{LogStorage, MetricStorage, TraceStorage},
    tasks::home_generator::HomeGeneratorTask,
    templating::TemplateEngine,
};
use std::collections::HashMap;
use std::{process, sync::Arc};

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

    config.validate()?;

    let metric_storage = Arc::new(MetricStorage::new());
    let trace_storage = Arc::new(TraceStorage::new());
    let log_storage = Arc::new(LogStorage::new());

    add_sample_data(&metric_storage, &trace_storage, &log_storage)?;

    let template_engine = Arc::new(TemplateEngine::new(&config.templates_dir));

    let scheduler = Scheduler::new(config.clone());

    let home_generator_task = Arc::new(HomeGeneratorTask::new(
        template_engine,
        metric_storage,
        trace_storage,
        log_storage,
        config.output_dir.to_string_lossy().into_owned(),
    ));

    scheduler.add_task(home_generator_task).await;

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

fn add_sample_data(
    metric_storage: &MetricStorage,
    trace_storage: &TraceStorage,
    log_storage: &LogStorage,
) -> Result<()> {
    let mut cpu_labels = HashMap::new();
    cpu_labels.insert("unit".to_string(), "%".to_string());
    cpu_labels.insert("trend".to_string(), "+2.3".to_string());

    let mut memory_labels = HashMap::new();
    memory_labels.insert("unit".to_string(), "GB".to_string());
    memory_labels.insert("trend".to_string(), "-0.5".to_string());

    metric_storage.add(Metric::new("CPU Usage", 78.5).with_labels(cpu_labels))?;
    metric_storage.add(Metric::new("Memory Usage", 4.2).with_labels(memory_labels))?;
    metric_storage.add(Metric::new("Response Time", 120.0).with_label("unit", "ms"))?;

    let api_trace = Trace::new("API Request", 157)
        .with_metadata("endpoint", "/api/users")
        .with_metadata("method", "GET")
        .with_metadata("status", "200");

    let db_trace = Trace::new("Database Query", 45)
        .with_metadata("query", "SELECT * FROM users")
        .with_metadata("rows", "250");

    trace_storage.add(api_trace)?;
    trace_storage.add(db_trace)?;

    log_storage.add(LogEntry::new(
        "Server started",
        LogLevel::Info,
        "app_server",
    ))?;
    log_storage.add(LogEntry::new(
        "Database connection established",
        LogLevel::Info,
        "database",
    ))?;
    log_storage.add(LogEntry::new(
        "Processing request: GET /api/users",
        LogLevel::Debug,
        "api",
    ))?;
    log_storage.add(LogEntry::new(
        "Cache miss for user data",
        LogLevel::Warning,
        "cache",
    ))?;

    info!("Added sample data for testing");
    Ok(())
}
