use serde_json::Error;
use ssm_lib::session::SessionBuilder;

use crate::args::StartSessionParams;

#[derive(Debug)]
pub enum Command {
    ReportInstallSuccess,
    Version,
    StartSession(StartSessionParams),
}

impl Command {
    pub fn execute(self) -> Result<(), Error> {
        match self {
            Command::ReportInstallSuccess => report_install_success(),
            Command::Version => report_version(),
            Command::StartSession(args) => start_session(args)?,
        };
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
fn start_session(args: StartSessionParams) -> Result<(), Error> {
    let _session = SessionBuilder::new()
        .with_stream_url(args.response.stream_url)
        .build();

    // TODO: Implement the rest of the session creation logic
    // TODO: Implement session handling logic
    Ok(())
}
