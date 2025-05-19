use std::collections::BTreeMap;

use beef::Cow;
use chrono::Local;
use colored::Colorize;
use log::kv::{Key, Value, VisitSource};
use log::{Level, LevelFilter, kv};

struct Collect<'kvs>(BTreeMap<Cow<'kvs, str>, Cow<'kvs, str>>);

impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
  fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
    self
      .0
      .insert(key.to_string().into(), value.to_string().into());
    Ok(())
  }
}

use crate::error::ConfigError;
use miette::Result;

pub fn setup_logger(verbosity: u8) -> Result<()> {
  let log_level = match verbosity {
    0 => LevelFilter::Error,
    1 => LevelFilter::Warn,
    2 => LevelFilter::Info,
    3 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  };

  // Gum log colors
  const TRACE_RGB: (u8, u8, u8) = (144, 144, 144);
  const DEBUG_RGB: (u8, u8, u8) = (95, 96, 255);
  const INFO_RGB: (u8, u8, u8) = (99, 254, 218);
  const WARN_RGB: (u8, u8, u8) = (219, 254, 143);
  const ERROR_RGB: (u8, u8, u8) = (254, 95, 136);

  fern::Dispatch::new()
    .format(move |out, message, record| {
      let time = Local::now().format("%H:%M");
      let lvl_plain = format!("{:>5}", record.level());
      let (r, g, b) = match record.level() {
        Level::Trace => TRACE_RGB,
        Level::Debug => DEBUG_RGB,
        Level::Info => INFO_RGB,
        Level::Warn => WARN_RGB,
        Level::Error => ERROR_RGB,
      };
      let lvl_colored = lvl_plain.truecolor(r, g, b);

      let has_kvs = record.key_values().count() > 0;
      if has_kvs {
        let mut visitor = Collect(BTreeMap::new());
        let _ = record.key_values().visit(&mut visitor);
        let collected = visitor.0;
        let (single, multiline) = collected
          .iter()
          .partition::<Vec<_>, _>(|(_, v)| !v.to_string().contains('\n'));

        // check if theres a key named SCOPE, if so pop it out and print infront of message
        let scope = single
          .iter()
          .find(|(k, _)| *k == "SCOPE")
          .map(|(k, v)| (k.to_string(), v.to_string()));

        let formatted_pairs = single
          .iter()
          .filter(|(k, _)| *k != "SCOPE")
          .map(|(k, v)| {
            let k = k.to_string().as_str().truecolor(142, 142, 142);
            let eq = "=".truecolor(142, 142, 142);
            let v = v.to_string();
            format!("{k}{eq}{v}")
          })
          .collect::<Vec<_>>()
          .join(" ");

        let formatted_multiline_pairs = multiline
          .iter()
          .map(|(k, v)| {
            let k = k.to_string().as_str().truecolor(142, 142, 142);
            let eq = "=".truecolor(142, 142, 142);
            let vb = "┊".truecolor(142, 142, 142);
            let v = v.to_string();

            format!(
              "{k}{eq}\n  {vb} {}",
              v.to_string()
                .lines()
                .collect::<Vec<_>>()
                .join(format!("\n  {vb} ").as_str())
            )
          })
          .collect::<Vec<_>>()
          .join("\n  ");

        out.finish(format_args!(
          "{time} {lvl_colored} {}{message}{}{}",
          if let Some((_, v)) = scope {
            format!("[{}] ", v.bold())
          } else {
            "".to_string()
          },
          if !single.is_empty() {
            format!(" {}", formatted_pairs)
          } else {
            "".to_string()
          },
          if !multiline.is_empty() {
            format!("\n  {}", formatted_multiline_pairs)
          } else {
            "".to_string()
          }
        ))
      } else {
        out.finish(format_args!("{time} {lvl_colored} {message}"))
      }
    })
    .level(log_level)
    .chain(std::io::stdout())
    .apply()
    .map_err(|err| ConfigError::LoggerSetup { source: err })?;

  //   let tracing_level = match verbosity {
  //   0 => tracing::Level::ERROR,
  //   1 => tracing::Level::WARN,
  //   2 => tracing::Level::INFO,
  //   3 => tracing::Level::DEBUG,
  //   _ => tracing::Level::TRACE,
  // };
  // let subscriber = FmtSubscriber::builder()
  //   .with_max_level(tracing_level)
  //   .finish();
  // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  Ok(())
}
