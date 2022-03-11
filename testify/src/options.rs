use std::str::FromStr;
use clap::Parser;

pub const ENV_CRATE_NAME_KEY: &'static str = "RU_CRATE_NAME";
pub const ENV_CRATE_ROOT_KEY: &'static str = "RU_CRATE_ROOT";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(allow_external_subcommands = true)]
pub struct RuConfig {
  #[clap(long = "ru-crate-root")]
  pub crate_root: String,
  #[clap(long = "ru-crate-name")]
  pub crate_name: String,
}

impl RuConfig {
  pub fn env_crate_name() -> String {
    std::env::var(ENV_CRATE_NAME_KEY)
        .expect("Crate name env var is not defined")
  }

  pub fn env_crate_root() -> String {
    std::env::var(ENV_CRATE_ROOT_KEY)
        .expect("Crate root env var is not defined")
  }
}