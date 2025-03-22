use std::{env, mem};

use serde_json::Value;

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
    #[allow(dead_code)]
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
                5 => start_session_args.target = Self::process_target_arg(&args[5])?,
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

    fn process_target_arg(arg: &str) -> Result<String, Error> {
        let target: Value = serde_json::from_str(arg)?;

        let target = match target {
            Value::Object(obj) => obj,
            _ => Err(Error::InvalidStartSessionObject)?,
        };

        let target = target
            .get("Target")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or(Error::InvalidStartSessionObject)?;

        Ok(target)
    }
}

#[cfg(test)]
mod test {
    const SESSION_MANAGER_PLUGIN: &str = "session-manager-plugin";
    #[test]
    fn validate_input_with_no_input_argument() {
        let args = vec![SESSION_MANAGER_PLUGIN.to_string()];
        let result = super::validate_args(args);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            super::Command::ReportInstallSuccess
        ));
    }

    #[test]
    fn validate_args_with_wrong_input_argument() {
        let wrong_argument = "wrong-argument".to_string();
        let args = vec![SESSION_MANAGER_PLUGIN.to_string(), wrong_argument.clone()];
        let result = super::validate_args(args);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            super::Error::UnknownOperation(arg) if arg == wrong_argument
        ));
    }

    #[test]
    fn validate_input() {
        let session_response = "{\"SessionId\": \"user-012345\", \"TokenValue\": \"ABCD\", \"StreamUrl\": \"wss://ssmmessages.us-east-1.amazonaws.com/v1/data-channel/user-012345?role=publish_subscribe\"}";
        let region = "us-east-1";
        let operation_name = "StartSession";
        let target = "i-0123abc";
        let ssm_endpoint = "https://ssm.us-east-1.amazonaws.com";
        let args = vec![
            SESSION_MANAGER_PLUGIN.to_string(),
            session_response.to_string(),
            region.to_string(),
            operation_name.to_string(),
            "".to_string(),
            format!("{{\"Target\": \"{target}\"}}"),
            ssm_endpoint.to_string(),
        ];
        let result = super::validate_args(args);
        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(matches!(result, super::Command::StartSession(_)));

        let args = match result {
            super::Command::StartSession(args) => args,
            _ => unreachable!("Already checked that the result is StartSession"),
        };

        assert_eq!(args.response, session_response.as_bytes());
        assert!(!args.is_aws_cli_upgrade_needed);
        assert_eq!(args.region, region);
        assert_eq!(args.operation_name, operation_name);
        assert!(args.profile.is_empty());
        assert_eq!(args.target, target);
        assert_eq!(args.ssm_endpoint, ssm_endpoint);
    }

    #[test]
    fn validate_input_with_env_variable_parameter() {
        let session_response_env_var = "AWS_SSM_START_SESSION_RESPONSE";
        let session_response = "{\"SessionId\": \"user-012345\", \"TokenValue\": \"Session-Token\", \"StreamUrl\": \"wss://ssmmessages.us-east-1.amazonaws.com/v1/data-channel/user-012345?role=publish_subscribe\"}";

        unsafe {
            std::env::set_var(session_response_env_var, session_response);
        }

        let region = "us-east-1";
        let operation_name = "StartSession";
        let target = "i-0123abc";
        let ssm_endpoint = "https://ssm.us-east-1.amazonaws.com";

        let args = vec![
            SESSION_MANAGER_PLUGIN.to_string(),
            session_response_env_var.to_string(),
            region.to_string(),
            operation_name.to_string(),
            "".to_string(),
            format!("{{\"Target\": \"{target}\"}}"),
            ssm_endpoint.to_string(),
        ];

        let result = super::validate_args(args);
        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(matches!(result, super::Command::StartSession(_)));

        let args = match result {
            super::Command::StartSession(args) => args,
            _ => unreachable!("Already checked that the result is StartSession"),
        };

        assert_eq!(args.response, session_response.as_bytes());
        assert!(!args.is_aws_cli_upgrade_needed);
        assert_eq!(args.region, region);
        assert_eq!(args.operation_name, operation_name);
        assert!(args.profile.is_empty());
        assert_eq!(args.target, target);
        assert_eq!(args.ssm_endpoint, ssm_endpoint);

        unsafe {
            std::env::remove_var(session_response_env_var);
        }
    }
}
