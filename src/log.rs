use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter,
    Layer,
    field::MakeExt,
    fmt::time::ChronoLocal,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::errors::*;

/// Install color_eyre and tracing with a default log level of INFO.
///
/// Can be overruled via the `LOGLEVEL=2` variable.
pub fn install_tracing() -> Result<()> {
    color_eyre::install()?;

    let log_level = std::env::var("LOGLEVEL").unwrap_or("1".into());
    let level = match log_level.as_str() {
        "0" => LevelFilter::WARN,
        "1" => LevelFilter::INFO,
        "2" => LevelFilter::DEBUG,
        "3" => LevelFilter::TRACE,
        _ => bail!("Found unexpected log level {log_level}"),
    };

    // tries to find local offset internally
    let timer = ChronoLocal::new("%H:%M:%S".into());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .map_fmt_fields(|f| f.debug_alt())
        .with_timer(timer)
        .with_writer(std::io::stderr);

    let filter_layer = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env()
        .wrap_err("RUST_LOG env variable is invalid")?;

    tracing_subscriber::Registry::default()
        .with(fmt_layer.with_filter(filter_layer))
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(())
}
