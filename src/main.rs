use anyhow::Result;
use clap::Parser;
use currency_conversion_cli::{cli::CliArgs, config::Config, errors::errors_handling, run};
use tracing_log::AsTrace;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments

    use anyhow::Context;
    use currency_conversion_cli::{cli::SubCommand, commands::config::prompt_and_store_config};
    let args = CliArgs::parse();

    // Initialize trace
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    // Get config
    let mut config: Config = match &args.config_path.is_none() {
        true => confy::load("currency-conversion-cli", args.config_profile.as_deref())
            .with_context(|| "Use \"currency-conversion-cli config\" to create the config")?,
        false => confy::load_path(args.config_path.clone().unwrap())
            .with_context(|| "Use \"currency-conversion-cli config\" to create the config")?,
    };

    // Initialized config if not
    if config.api_key == "#INSERT_API_KEY_HERE#" {
        config =
            prompt_and_store_config(&config, &args.config_path, args.config_profile.as_deref())?;

        if let SubCommand::Config = args.sub_command {
            return Ok(());
        }
    }

    match run(
        args.sub_command,
        config,
        args.config_path,
        args.config_profile.as_deref(),
    )
    .await
    {
        Err(error) => errors_handling(error),
        Ok(()) => Ok(()),
    }
}
