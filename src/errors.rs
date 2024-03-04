#[derive(Debug)]
pub enum CustomError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::ReqwestError(err) => write!(f, "Error sending request: {}", err),
            CustomError::SerdeJsonError(err) => write!(f, "Error parsing JSON: {}", err),
        }
    }
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> Self {
        CustomError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(err: serde_json::Error) -> Self {
        CustomError::SerdeJsonError(err)
    }
}

pub fn handle_serde_json_error(err: serde_json::Error) -> CustomError {
    CustomError::SerdeJsonError(err)
}

pub fn handle_erqwest_error(err: reqwest::Error) -> CustomError {
    CustomError::ReqwestError(err)
}
