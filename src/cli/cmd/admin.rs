pub mod convert_all_pdfs;
pub mod disable_2fa;
pub mod generate_previews;
pub mod recreate_index;
pub mod reset_password;

use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};

/// Admin commands.
///
/// These commands require the admin secret from the server config
/// file.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    /// This secret is required to access them. If not given here, it
    /// is taken from the config file.
    #[clap(short, long)]
    pub admin_secret: Option<String>,

    #[clap(subcommand)]
    pub subcmd: AdminCommand,
}

#[derive(Debug, Snafu)]
pub enum Error {
    GeneratePreview { source: generate_previews::Error },
    RecreateIndex { source: recreate_index::Error },
    ResetPassword { source: reset_password::Error },
    ConvertAllPdfs { source: convert_all_pdfs::Error },
    Disable2FA { source: disable_2fa::Error },
}

#[derive(Parser, Debug)]
pub enum AdminCommand {
    #[clap(version)]
    GeneratePreviews(generate_previews::Input),

    #[clap(version)]
    RecreateIndex(recreate_index::Input),

    #[clap(version)]
    ResetPassword(reset_password::Input),

    #[clap(version)]
    ConvertAllPdfs(convert_all_pdfs::Input),

    #[clap(name = "disable-2fa")]
    #[clap(version)]
    Disable2FA(disable_2fa::Input),
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        match &self.subcmd {
            AdminCommand::GeneratePreviews(input) => {
                input.exec(self, ctx).context(GeneratePreviewSnafu)
            }
            AdminCommand::RecreateIndex(input) => input.exec(self, ctx).context(RecreateIndexSnafu),
            AdminCommand::ResetPassword(input) => input.exec(self, ctx).context(ResetPasswordSnafu),
            AdminCommand::ConvertAllPdfs(input) => {
                input.exec(self, ctx).context(ConvertAllPdfsSnafu)
            }
            AdminCommand::Disable2FA(input) => input.exec(self, ctx).context(Disable2FASnafu),
        }
    }
}

pub trait AdminCmd {
    type CmdError;

    fn exec<'a>(&self, admin_opts: &'a Input, args: &'a Context) -> Result<(), Self::CmdError>;
}

fn get_secret(opts: &Input, ctx: &Context) -> Option<String> {
    let secret = opts
        .admin_secret
        .as_ref()
        .or_else(|| ctx.cfg.admin_secret.as_ref())
        .map(String::clone);

    if secret.is_some() && ctx.opts.verbose > 2 {
        log::debug!("Using secret: {:?}", secret);
    }

    secret
}
