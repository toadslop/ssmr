use crate::args::StartSessionParams;
use ssm_lib::session::SessionBuilder;

#[derive(Debug)]
pub enum Command {
    ReportInstallSuccess,
    Version,
    StartSession(StartSessionParams),
}

impl Command {
    pub async fn execute(self) -> Result<(), crate::Error> {
        match self {
            Command::ReportInstallSuccess => report_install_success(),
            Command::Version => report_version(),
            Command::StartSession(args) => start_session(args).await?,
        }
        Ok(())
    }
}

fn report_install_success() {
    println!(
        "\nThe Session Manager plugin was installed successfully. Use the AWS CLI to start a session.\n\n"
    );
}

fn report_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

#[allow(clippy::unnecessary_wraps)]
async fn start_session(args: StartSessionParams) -> Result<(), crate::Error> {
    // Allow deprecated usage of `with_aws_cli_upgrade_needed` for compatibility with the original implementation.
    #[allow(deprecated)]
    let session = SessionBuilder::new()
        .with_stream_url(args.response.stream_url)
        .with_endpoint(args.ssm_endpoint)
        .with_aws_cli_upgrade_needed(args.is_aws_cli_upgrade_needed)
        .with_session_id(args.response.session_id)
        .with_target_id(args.target)
        .build();

    session.execute().await?;

    // TODO: Implement the rest of the session creation logic
    // TODO: Implement session handling logic
    Ok(())
}
