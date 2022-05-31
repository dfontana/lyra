use std::fs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use super::init_home;

pub fn init_logs() -> Result<(), anyhow::Error> {
  tracing::subscriber::with_default(
    FmtSubscriber::builder()
      .with_max_level(Level::INFO)
      .finish(),
    || -> Result<(), anyhow::Error> {
      let logs_dir = init_home()?.join("logs");
      if !logs_dir.exists() {
        info!(
          "Log dir missing, generating default at {}",
          logs_dir.to_string_lossy()
        );
        fs::create_dir_all(&logs_dir)?;
      }
      tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
          .with_max_level(Level::INFO)
          .with_writer(tracing_appender::rolling::daily(logs_dir, "lyra.log"))
          .finish(),
      )
      .expect("setting default subscriber failed");
      Ok(())
    },
  )
}
