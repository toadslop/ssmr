use std::{env, mem};

use crate::{LEGACY_ARGUMENT_LENGTH, command::Command, error::Error};

pub fn validate_args(mut args: Vec<String>) -> Result<Command, Error> {
    let command = match args.len() {
        1 => Command::ReportInstallSuccess,
        2 if args[1] == "--version" => Command::Version,
        count if (2..LEGACY_ARGUMENT_LENGTH).contains(&count) => {
            Err(Error::UnknownOperation(mem::take(&mut args[1])))?
        }
        _ => {
            let args = StartSessionArgs::validate_args(args)?;
            Command::StartSession(args)
        }
    };

    Ok(command)
}

#[derive(Debug, Default)]
pub struct StartSessionArgs {
    pub is_aws_cli_upgrade_needed: bool,
    pub response: Vec<u8>,
    pub region: String,
    pub operation_name: String,
    pub profile: String,
    pub target: String,
    pub ssm_endpoint: String,
}

impl StartSessionArgs {
    const AWS_SSM_START_SESSION_RESPONSE: &str = "AWS_SSM_START_SESSION_RESPONSE";

    pub fn validate_args(mut args: Vec<String>) -> Result<Self, Error> {
        let mut start_session_args = StartSessionArgs {
            is_aws_cli_upgrade_needed: args.len() == LEGACY_ARGUMENT_LENGTH,
            ..Default::default()
        };

        for i in 1..args.len() {
            match i {
                1 => start_session_args.response = Self::process_response_arg(&args[1])?,
                2 => start_session_args.region = mem::take(&mut args[2]),
                3 => start_session_args.operation_name = mem::take(&mut args[3]),
                4 => start_session_args.profile = mem::take(&mut args[4]),
                5 => start_session_args.target = todo!("deserialize target"),
                6 => start_session_args.ssm_endpoint = mem::take(&mut args[6]),
                _ => Err(Error::IncorrectNumArgs)?,
            }
        }

        Ok(start_session_args)
    }

    fn process_response_arg(arg: &str) -> Result<Vec<u8>, Error> {
        let response = match arg.starts_with(Self::AWS_SSM_START_SESSION_RESPONSE) {
            false => arg.as_bytes().to_vec(),
            true => {
                let response = env::var(arg)?;
                unsafe { env::remove_var(arg) };
                response.into_bytes()
            }
        };

        Ok(response)
    }
}
