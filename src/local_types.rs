use crate::errors;

pub type ResultSerde = Result<serde_json::Value, errors::CustomError>;
