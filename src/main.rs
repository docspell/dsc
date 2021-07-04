use dsc::cmd::Cmd;
use dsc::config::DsConfig;
use dsc::opts::SubCommand;
use log;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    let opts = dsc::read_args();
    let cfg: DsConfig = dsc::read_config(&opts.config).expect("Config could not be read");

    match opts.subcmd {
        SubCommand::Version(input) => {
            log::info!("Running version: {:?}", input);
            input.exec(cfg).expect("Command failed");
        }
    }
}
