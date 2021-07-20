pub mod generate_previews;
pub mod recreate_index;
pub mod reset_password;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use clap::{AppSettings, Clap};

/// Admin commands.
///
/// These commands require the admin secret from the server config
/// file.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// This secret is required to access them. If not given here, it
    /// is taken from the config file.
    #[clap(short, long)]
    pub admin_secret: Option<String>,

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
            AdminCommand::GeneratePreviews(input) => input.exec(self, args),
            AdminCommand::RecreateIndex(input) => input.exec(self, args),
            AdminCommand::ResetPassword(input) => input.exec(self, args),
        }
    }
}

pub trait AdminCmd {
    fn exec<'a>(&self, admin_opts: &'a Input, args: &'a CmdArgs) -> Result<(), CmdError>;
}

fn get_secret(opts: &Input, args: &CmdArgs) -> Option<String> {
    let secret = opts
        .admin_secret
        .as_ref()
        .or(args.cfg.admin_secret.as_ref())
        .map(String::clone);

    if secret.is_some() && args.opts.verbose > 2 {
        log::debug!("Using secret: {:?}", secret);
    }

    secret
}
