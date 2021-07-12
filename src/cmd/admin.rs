pub mod generate_previews;
pub mod recreate_index;
pub mod reset_password;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use clap::{AppSettings, Clap};

/// [Admin] Commands that require the admin secret from the server
/// config file.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(subcommand)]
    pub subcmd: AdminCommand,
}

#[derive(Clap, Debug)]
pub enum AdminCommand {
    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    GeneratePreviews(generate_previews::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    RecreateIndex(recreate_index::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    ResetPassword(reset_password::Input),
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        match &self.subcmd {
            AdminCommand::GeneratePreviews(input) => input.exec(args),
            AdminCommand::RecreateIndex(input) => input.exec(args),
            AdminCommand::ResetPassword(input) => input.exec(args),
        }
    }
}

fn get_secret(args: &CmdArgs) -> Option<String> {
    let secret = args.admin_secret();

    if secret.is_some() && args.opts.verbose > 2 {
        log::debug!("Using secret: {:?}", secret);
    }

    secret
}
