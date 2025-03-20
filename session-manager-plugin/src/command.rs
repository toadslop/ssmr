use crate::args::StartSessionArgs;

#[derive(Debug)]
pub enum Command {
    ReportInstallSuccess,
    Version,
    StartSession(StartSessionArgs),
}

impl Command {
    pub fn execute(self) {
        match self {
            Command::ReportInstallSuccess => report_install_success(),
            Command::Version => report_version(),
            Command::StartSession(args) => start_session(args),
        }
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

fn start_session(_args: StartSessionArgs) {
    todo!()
}
