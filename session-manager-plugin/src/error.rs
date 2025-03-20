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
}
