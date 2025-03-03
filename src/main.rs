// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use upstreams_rs::{host, versioning::VersionExtractor};

/// Configures the tracing infrastructure with appropriate formatting and filtering
///
/// Sets up tracing with ANSI colors, uptime timer, and target information.
/// Uses environment variables for filtering or defaults to trace level.
fn configure_tracing() -> color_eyre::Result<()> {
    let f = Format::default()
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(false);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().event_format(f))
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_tracing()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ext = VersionExtractor::new()?;
    for arg in args {
        let version = ext.extract(&arg)?;
        eprintln!("name = {}, version = {}", version.name, version.version);

        let url = url::Url::parse(&arg)?;
        let host = host::from_url(&url)?;
        let versions = host.versions().await?;

        let c = colored_json::to_colored_json_auto(&versions)?;
        println!("{}", c);
    }
    Ok(())
}
