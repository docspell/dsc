use crate::opts::Format;
use prettytable::Table;
use serde::Serialize;
use snafu::Snafu;
use std::convert::From;

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

pub trait AsTable {
    fn to_table(&self) -> Table;
}

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

pub fn str_or_empty(opt: &Option<String>) -> &str {
    opt.as_ref().map(|s| s.as_str()).unwrap_or("")
}
