//! A [`Sink`] is a common way to output datastructures for a command.
//!
//! A command may format its data based on the common `--format`
//! option. Data types that should be presented in this way can
//! implement the [`Sink`] trait so commands can easily output them.
//!
//! If a type implements serdes Serialize trait and the
//! [`super::table::AsTable`] trait, a Sink is implemented for free.

use super::opts::Format;
use crate::cli::table::AsTable;
use serde::Serialize;
use snafu::Snafu;
use std::convert::From;

/// Defines different outputs for a data type given via a [`Format`]
/// argument.
///
/// The formats `json` and `lisp` are handled via
/// [serde](https://serde.rs), the formats `tabular` and `csv` are
/// handled by [prettytable](https://crates.io/crates/prettytable-rs)
/// (and the csv crate).
pub trait Sink
where
    Self: Serialize + AsTable,
{
    fn write_value(format: Format, value: &Self) -> Result<(), Error> {
        match format {
            Format::Json => {
                serde_json::to_writer(std::io::stdout(), &value)?;
                Ok(())
            }
            Format::Lisp => {
                serde_lexpr::to_writer(std::io::stdout(), &value)?;
                Ok(())
            }
            Format::Csv => Self::write_csv(value),
            Format::Tabular => Self::write_tabular(value),
        }
    }

    fn write_tabular(value: &Self) -> Result<(), Error> {
        let table = value.to_table();
        table.printstd();
        Ok(())
    }

    fn write_csv(value: &Self) -> Result<(), Error> {
        let table = value.to_table();
        table.to_csv(std::io::stdout())?;
        Ok(())
    }
}

/// Possible errors when serializing data.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error serializing to JSON"))]
    Json { source: serde_json::Error },

    #[snafu(display("Error serializing to Lisp"))]
    Lisp { source: serde_lexpr::Error },

    #[snafu(display("Error serializing to CSV"))]
    Csv { source: csv::Error },
}
impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Error {
        Error::Csv { source: e }
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json { source: e }
    }
}
impl From<serde_lexpr::Error> for Error {
    fn from(e: serde_lexpr::Error) -> Error {
        Error::Lisp { source: e }
    }
}
