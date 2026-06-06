use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// 初始化日志系统，同时输出到控制台和按日滚动的日志文件
pub fn init(app_data_dir: &std::path::Path) {
    let log_dir = app_data_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {e}");
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "desk-manager.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    std::mem::forget(_guard);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_thread_ids(false);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Desk Manager logging initialized");
}
