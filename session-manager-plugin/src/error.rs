#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "\nUnknown operation {0}. \nUse session-manager-plugin --version to check the version.\n\n"
    )]
    UnknownOperation(String),
    #[error("Error reading environment variable: {0}")]
    EnvVar(#[from] std::env::VarError),
    #[error("Found more args than expected")]
    IncorrectNumArgs,
    #[error("Failed to deserialize start session args: {0}")]
    ArgDeserializatonFailure(#[from] serde_json::Error),
    #[error("Expected start session args to be a JSON object with key Target")]
    InvalidStartSessionObject,
    #[error("Expected response for start session to be a JSON object")]
    InvalidStartSessionResponseObject,
    #[error("Session execution failed: {0}")]
    ExecuteSessionFailure(#[from] ssm_lib::error::Error),
}
