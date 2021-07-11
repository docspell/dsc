use crate::opts::Format;
use serde::Serialize;
use std::convert::From;

pub trait Sink
where
    Self: Serialize,
{
    fn write_value(format: Format, value: &Self) -> Result<(), SerError> {
        match format {
            Format::Json => {
                let txt = serde_json::to_string(&value).map_err(SerError::Json)?;
                println!("{}", txt);
                Ok(())
            }
            Format::Lisp => {
                let txt = serde_lexpr::to_string(&value).map_err(SerError::Lisp)?;
                println!("{}", txt);
                Ok(())
            }
            Format::Csv => Self::write_csv(value),
            Format::Tabular => Self::write_tabular(value),
        }
    }

    fn write_tabular(value: &Self) -> Result<(), SerError>;

    fn write_csv(value: &Self) -> Result<(), SerError>;

    fn str_or_empty(opt: &Option<String>) -> &str {
        opt.as_ref().map(|s| s.as_str()).unwrap_or("")
    }
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
