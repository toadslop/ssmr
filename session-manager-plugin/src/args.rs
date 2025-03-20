use crate::{LEGACY_ARGUMENT_LENGTH, command::Command, error::Error};

pub fn validate_args(args: &[String]) -> Result<Command, Error> {
    let command = if args.len() == 1 {
        Command::ReportInstallSuccess
    } else if args.len() == 2 && args[1] == "--version" {
        Command::Version
    } else if args.len() >= 2 && args.len() < LEGACY_ARGUMENT_LENGTH {
        Err(Error::UnknownOperation(args[1].clone()))?
    } else {
        let args = StartSessionArgs::validate_args(args)?;
        Command::StartSession(args)
    };

    Ok(command)
}

#[derive(Debug)]
pub struct StartSessionArgs {
    pub is_aws_cli_upgrade_needed: bool,
}

impl StartSessionArgs {
    pub fn validate_args(args: &[String]) -> Result<Self, Error> {
        if args.len() >= 2 && args.len() < LEGACY_ARGUMENT_LENGTH {
            Err(Error::UnknownOperation(args[1].clone()))?
        }

        let is_aws_cli_upgrade_needed = args.len() == LEGACY_ARGUMENT_LENGTH;

        Ok(StartSessionArgs {
            is_aws_cli_upgrade_needed,
        })
    }
}
