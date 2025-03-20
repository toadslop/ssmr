#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "\nUnknown operation {0}. \nUse session-manager-plugin --version to check the version.\n\n"
    )]
    UnknownOperation(String),
}
