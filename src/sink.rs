use crate::opts::Format;
use prettytable::Table;
use serde::Serialize;
use std::convert::From;

pub trait Sink
where
    Self: Serialize + AsTable,
{
    fn write_value(format: Format, value: &Self) -> Result<(), SerError> {
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

    fn write_tabular(value: &Self) -> Result<(), SerError> {
        let table = value.to_table();
        table.printstd();
        Ok(())
    }

    fn write_csv(value: &Self) -> Result<(), SerError> {
        let table = value.to_table();
        table.to_csv(std::io::stdout())?;
        Ok(())
    }
}

pub trait AsTable {
    fn to_table(&self) -> Table;
}

#[derive(Debug)]
pub enum SerError {
    Json(serde_json::Error),
    Lisp(serde_lexpr::Error),
    Csv(csv::Error),
    IO(std::io::Error),
}
impl From<std::io::Error> for SerError {
    fn from(e: std::io::Error) -> SerError {
        SerError::IO(e)
    }
}
impl From<csv::Error> for SerError {
    fn from(e: csv::Error) -> SerError {
        SerError::Csv(e)
    }
}
impl From<serde_json::Error> for SerError {
    fn from(e: serde_json::Error) -> SerError {
        SerError::Json(e)
    }
}
impl From<serde_lexpr::Error> for SerError {
    fn from(e: serde_lexpr::Error) -> SerError {
        SerError::Lisp(e)
    }
}

pub fn str_or_empty(opt: &Option<String>) -> &str {
    opt.as_ref().map(|s| s.as_str()).unwrap_or("")
}
