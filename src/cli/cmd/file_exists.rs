use clap::{Clap, ValueHint};
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

use crate::cli::opts::EndpointOpts;
use crate::cli::sink::Error as SinkError;
use crate::http::payload::CheckFileResult;
use crate::http::Error as HttpError;
use crate::util::digest;

use super::{Cmd, Context};

/// Checks if the given files exist in docspell.
///
/// To check a file, an authenticated user is required, a source id or
/// the secret to the integration endpoint. The latter allows to check
/// across collectives.
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// One or more files to check
    #[clap(required = true, min_values = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Collective must be present when using integration endpoint."))]
    NoCollective,

    #[snafu(display("Calculating digest of file {} failed: {}", path.display(), source))]
    DigestFail {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let mut results = Vec::with_capacity(self.files.capacity());
        for file in &self.files {
            if file.is_file() {
                let result = check_file(&file, &self.endpoint, ctx)?;
                results.push(result);
            } else {
                log::debug!("Ignoring directory: {}", file.display());
            }
        }
        ctx.write_result(results).context(WriteResult)?;
        Ok(())
    }
}

pub fn check_file(
    file: &PathBuf,
    opts: &EndpointOpts,
    ctx: &Context,
) -> Result<CheckFileResult, Error> {
    let fa = opts.to_file_auth(ctx);
    let hash = digest::digest_file_sha256(file).context(DigestFail { path: file })?;
    let mut result = ctx.client.file_exists(hash, &fa).context(HttpClient)?;
    result.file = file.canonicalize().ok().map(|p| p.display().to_string());
    Ok(result)
}
