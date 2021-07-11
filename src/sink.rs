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
            Format::Tabular => Self::write_tabular(value),
        }
    }

    fn write_tabular(value: &Self) -> Result<(), SerError>;
}

#[derive(Debug)]
pub enum SerError {
    Json(serde_json::Error),
    Lisp(serde_lexpr::Error),
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
