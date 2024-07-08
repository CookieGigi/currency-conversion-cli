use anyhow::Result;
use cli::SubCommand;
use commands::update::run_update;
use config::Config;

pub mod cli;
pub mod commands;
pub mod config;
pub mod errors;

/// Handle commands execution
#[cfg(not(tarpaulin_include))]
pub async fn run(
    sub_command: SubCommand,
    config: Config,
    config_path: Option<String>,
    config_profile: Option<&str>,
) -> Result<()> {
    use commands::convert::run_convert;

    use crate::commands::{config::run_config, info::run_info, list::run_list};

    match sub_command {
        SubCommand::Update(args) => run_update(&config, &args).await?,
        SubCommand::Convert(args) => run_convert(&config, &args).await?,
        SubCommand::List(args) => run_list(&config, &args).await?,
        SubCommand::Info(args) => run_info(config, &args, config_path).await?,
        SubCommand::Config => run_config(&config, &config_path, config_profile)?,
    }
    Ok(())
}
