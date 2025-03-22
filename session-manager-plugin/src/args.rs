use serde_json::Value;
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
            let args = StartSessionParams::try_from_args(args)?;
            Command::StartSession(args)
        }
    };

    Ok(command)
}

#[derive(Debug, Default)]
pub struct StartSessionParams {
    #[allow(dead_code)] // TODO: remove
    pub is_aws_cli_upgrade_needed: bool,
    pub response: StartSessionOutput,
    pub region: String, // TODO: original implementation sets this to a global variable; need to evaluate how used and decide implementation
    pub operation_name: String,
    pub profile: String, // TODO: original implementation sets this to a global variable; need to evaluate how used and decide implementation
    pub target: String,
    pub ssm_endpoint: String,
}

impl StartSessionParams {
    const AWS_SSM_START_SESSION_RESPONSE: &str = "AWS_SSM_START_SESSION_RESPONSE";

    pub fn try_from_args(mut args: Vec<String>) -> Result<Self, Error> {
        let mut start_session_params = Self {
            is_aws_cli_upgrade_needed: args.len() == LEGACY_ARGUMENT_LENGTH,
            ..Default::default()
        };

        for i in 1..args.len() {
            match i {
                1 => Self::process_response_arg(&args[1], &mut start_session_params)?,
                2 => start_session_params.region = mem::take(&mut args[2]),
                3 => {
                    start_session_params.operation_name =
                        Self::process_opname(mem::take(&mut args[3]))?;
                }
                4 => start_session_params.profile = mem::take(&mut args[4]),
                5 => start_session_params.target = Self::process_target_arg(&args[5])?,
                6 => start_session_params.ssm_endpoint = mem::take(&mut args[6]),
                _ => Err(Error::IncorrectNumArgs)?,
            }
        }

        Ok(start_session_params)
    }

    fn process_opname(arg: String) -> Result<String, Error> {
        match arg.as_str() {
            "StartSession" => Ok(arg),
            _ => Err(Error::UnknownOperation(arg.to_string())),
        }
    }

    fn process_response_arg(
        arg: &str,
        start_session_params: &mut StartSessionParams,
    ) -> Result<(), Error> {
        let response = if arg.starts_with(Self::AWS_SSM_START_SESSION_RESPONSE) {
            let response = env::var(arg)?;
            unsafe { env::remove_var(arg) };
            response
        } else {
            arg.to_string()
        };

        let response: Value = serde_json::from_str(&response)?;

        let response = match response {
            Value::Object(obj) => obj,
            _ => Err(Error::InvalidStartSessionResponseObject)?,
        };

        start_session_params.response.session_id = response
            .get("SessionId")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or(Error::InvalidStartSessionResponseObject)?;

        start_session_params.response.token_value = response
            .get("TokenValue")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or(Error::InvalidStartSessionResponseObject)?;

        start_session_params.response.stream_url = response
            .get("StreamUrl")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or(Error::InvalidStartSessionResponseObject)?;

        Ok(())
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

#[derive(Debug, Default)]
pub struct StartSessionOutput {
    pub session_id: String,
    pub token_value: String,
    pub stream_url: String,
}

#[cfg(test)]
mod test {
    use std::env::VarError;

    static SESSION_MANAGER_PLUGIN: &str = "session-manager-plugin";
    static SESSION_ID: &str = "user-012345";
    static TOKEN_VALUE: &str = "ABCD";
    static SESSION_RESPONSE_ENV_VAR: &str = "AWS_SSM_START_SESSION_RESPONSE";
    static REGION: &str = "us-east-1";

    fn get_stream_url() -> String {
        format!(
            "wss://ssmmessages.us-east-1.amazonaws.com/v1/data-channel/{SESSION_ID}?role=publish_subscribe"
        )
    }

    fn get_session_response() -> String {
        let stream_url = get_stream_url();
        format!(
            "{{\"SessionId\": \"{SESSION_ID}\", \"TokenValue\": \"{TOKEN_VALUE}\", \"StreamUrl\": \"{stream_url}\"}}"
        )
    }

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
        let operation_name = "StartSession";
        let target = "i-0123abc";
        let ssm_endpoint = "https://ssm.us-east-1.amazonaws.com";
        let args = vec![
            SESSION_MANAGER_PLUGIN.to_string(),
            get_session_response(),
            REGION.to_string(),
            operation_name.to_string(),
            String::default(),
            format!("{{\"Target\": \"{target}\"}}"),
            ssm_endpoint.to_string(),
        ];
        let result = super::validate_args(args);
        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(matches!(result, super::Command::StartSession(_)));

        let super::Command::StartSession(args) = result else {
            unreachable!("Already checked that the result is StartSession")
        };

        assert_eq!(args.response.session_id, SESSION_ID);
        assert_eq!(args.response.stream_url, get_stream_url());
        assert_eq!(args.response.token_value, TOKEN_VALUE);
        assert!(!args.is_aws_cli_upgrade_needed);
        assert_eq!(args.region, REGION);
        assert_eq!(args.operation_name, operation_name);
        assert!(args.profile.is_empty());
        assert_eq!(args.target, target);
        assert_eq!(args.ssm_endpoint, ssm_endpoint);
    }

    #[test]
    fn validate_input_with_env_variable_parameter() {
        unsafe {
            std::env::set_var(SESSION_RESPONSE_ENV_VAR, get_session_response());
        }

        let operation_name = "StartSession";
        let target = "i-0123abc";
        let ssm_endpoint = "https://ssm.us-east-1.amazonaws.com";

        let args = vec![
            SESSION_MANAGER_PLUGIN.to_string(),
            SESSION_RESPONSE_ENV_VAR.to_string(),
            REGION.to_string(),
            operation_name.to_string(),
            String::default(),
            format!("{{\"Target\": \"{target}\"}}"),
            ssm_endpoint.to_string(),
        ];

        let result = super::validate_args(args);
        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(matches!(result, super::Command::StartSession(_)));

        let super::Command::StartSession(args) = result else {
            unreachable!("Already checked that the result is StartSession")
        };

        assert_eq!(args.response.session_id, SESSION_ID);
        assert_eq!(args.response.stream_url, get_stream_url());
        assert_eq!(args.response.token_value, TOKEN_VALUE);
        assert!(!args.is_aws_cli_upgrade_needed);
        assert_eq!(args.region, REGION);
        assert_eq!(args.operation_name, operation_name);
        assert!(args.profile.is_empty());
        assert_eq!(args.target, target);
        assert_eq!(args.ssm_endpoint, ssm_endpoint);

        unsafe {
            std::env::remove_var(SESSION_RESPONSE_ENV_VAR);
        }
    }

    #[test]
    fn validate_input_with_wrong_env_variable_name() {
        let wrong_env_name = "WRONG_ENV_NAME";
        unsafe {
            std::env::set_var(wrong_env_name, get_session_response());
        }

        let region = "us-east-1";
        let operation_name = "StartSession";
        let target = "i-0123abc";
        let ssm_endpoint = "https://ssm.us-east-1.amazonaws.com";

        let args = vec![
            SESSION_MANAGER_PLUGIN.to_string(),
            wrong_env_name.to_string(),
            region.to_string(),
            operation_name.to_string(),
            String::default(),
            format!("{{\"Target\": \"{target}\"}}"),
            ssm_endpoint.to_string(),
        ];

        let result = super::validate_args(args);
        dbg!(&result);
        assert!(result.is_err());

        let result = result.unwrap_err();

        assert!(matches!(result, super::Error::ArgDeserializatonFailure(_)));
        assert!(matches!(
            std::env::var(SESSION_RESPONSE_ENV_VAR),
            Err(VarError::NotPresent)
        ));

        unsafe {
            std::env::remove_var(wrong_env_name);
        }
    }
}
