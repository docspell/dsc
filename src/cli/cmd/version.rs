use crate::cli::sink::Sink;
use crate::cli::table::AsTable;
use crate::http::payload::{BuildInfo, VersionInfo};
use crate::http::Error as HttpError;
use clap::Parser;
use prettytable::{cell, row, Table};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;

/// Prints version about server and client.
///
/// Queries the server for its version information and prints more
/// version details about this client.
#[derive(Parser, Debug, PartialEq)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = ctx.client.version().context(HttpClientSnafu)?;
        let vinfo = AllVersion::default(result, ctx.base_url());
        ctx.write_result(vinfo).context(WriteResultSnafu)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct AllVersion {
    pub client: BuildInfo,
    pub server: VersionInfo,
    pub docspell_url: String,
}
impl AllVersion {
    pub fn default(server: VersionInfo, docspell_url: String) -> AllVersion {
        AllVersion {
            client: BuildInfo::default(),
            server,
            docspell_url,
        }
    }
}

impl AsTable for AllVersion {
    fn to_table(&self) -> Table {
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_CLEAN);
        let mut ct = self.client.to_table();
        ct.set_titles(row!["Client (dsc)", ""]);
        let mut st = self.server.to_table();
        st.set_titles(row!["Docspell Server", self.docspell_url]);
        table.add_row(row![st]);
        table.add_row(row![ct]);
        table
    }
}
impl Sink for AllVersion {}
