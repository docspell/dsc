pub mod generate_previews;
pub mod recreate_index;
pub mod reset_password;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::VERSION;
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
    #[clap(version = VERSION)]
    GeneratePreviews(generate_previews::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    RecreateIndex(recreate_index::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    ResetPassword(reset_password::Input),
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let secret = args
            .opts
            .admin_secret
            .as_ref()
            .ok_or(CmdError::AuthError("No admin secret provided".into()))?;
        if args.opts.verbose >= 2 {
            log::debug!("Using secret: {:}", secret);
        }
        match &self.subcmd {
            AdminCommand::GeneratePreviews(input) => input.exec(secret, args),
            AdminCommand::RecreateIndex(input) => input.exec(secret, args),
            AdminCommand::ResetPassword(input) => input.exec(secret, args),
        }
    }
}

pub trait AdminCmd {
    fn exec<'a>(&self, secret: &str, args: &'a CmdArgs) -> Result<(), CmdError>;
}
