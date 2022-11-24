use chrono::{NaiveDate, Utc};
use std::{cmp::Ordering, fs};
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
      } else {
        let today: NaiveDate = Utc::now().date_naive();
        info!("Clearing logs older than {}", today);
        logs_dir.read_dir().map(|rd| {
          rd.filter_map(Result::ok)
            .filter_map(|de| {
              let name = de.file_name();
              let lossy = name.to_string_lossy();
              let trimmed = lossy.trim_start_matches("lyra.log.");
              NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .ok()
                .map(|nd| (de, nd))
            })
            .filter(|(_, nd)| nd.cmp(&today) == Ordering::Less)
            .for_each(|(de, dt)| {
              println!("Dropping log file: {}", dt);
              if let Err(e) = fs::remove_file(de.path()) {
                info!("Failed to delete old log file {} -> {:?}", dt, e)
              }
            })
        })?;
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
