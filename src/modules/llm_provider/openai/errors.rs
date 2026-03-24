use async_openai::error::OpenAIError;
use base64::DecodeError;
use serde_json::Error as SerdeError;
use std::io::Error as IOError;

use crate::modules::llm_provider::errors::GenerationsErrors;
use crate::modules::llm_provider::openai::models::errors::{OpenRouterError, OpenRouterErrorCodes};

impl From<OpenAIError> for GenerationsErrors {
    fn from(err: OpenAIError) -> Self {
        match err {
            OpenAIError::ApiError(_err) => {
                GenerationsErrors::ServiceUnavailable(format!("Api Error: {:?}", _err))
            }
            OpenAIError::InvalidArgument(err) => GenerationsErrors::AnotherError(err.to_string()),
            OpenAIError::FileReadError(err) | OpenAIError::FileSaveError(err) => {
                GenerationsErrors::IOError(err.to_string())
            }
            OpenAIError::Reqwest(err) => {
                GenerationsErrors::RequestError(format!("Request Error: {}", err))
            }
            OpenAIError::JSONDeserialize(context, err) => {
                let error: Result<OpenRouterError, _> = serde_json::from_str(&err);
                
                match error {
                    Ok(err) => {
                        let error_code = err.error().code();
                        let error_msg = err.error().message();
                        tracing::error!(error_msg);
                        match OpenRouterErrorCodes::from_status_code(error_code.to_owned()) {
                            Some(OpenRouterErrorCodes::Unauthorized) => {
                                GenerationsErrors::Unauthorized(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::ModelModeration) => {
                                GenerationsErrors::ModelModerationError(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::RateLimit) => {
                                GenerationsErrors::RateLimited(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::ServiceUnavailable) => {
                                GenerationsErrors::ServiceUnavailable(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::NoCredits) => {
                                GenerationsErrors::NoCredits(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::BadRequest) => {
                                GenerationsErrors::BadRequest(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::InvalidResponse) => {
                                GenerationsErrors::InvalidResponse(error_msg.to_owned())
                            }
                            Some(OpenRouterErrorCodes::Timeout) => {
                                GenerationsErrors::Timeout(error_msg.to_owned())
                            }

                            None => GenerationsErrors::AnotherError(format!(
                                "Unknown error code: {}",
                                error_msg
                            )),
                        }
                    }
                    Err(err) => GenerationsErrors::AnotherError(format!(
                        "Error on Deserialize request: context: {context},\n error: {err}",
                        context = context,
                        err = err
                    )),
                }
            }
            _ => GenerationsErrors::AnotherError(format!("OpenAI Generation Error: {}", err)),
        }
    }
}

impl From<DecodeError> for GenerationsErrors {
    fn from(err: DecodeError) -> Self {
        GenerationsErrors::IOError(format!("Error of decoding image: {}", err))
    }
}

impl From<IOError> for GenerationsErrors {
    fn from(err: IOError) -> Self {
        GenerationsErrors::IOError(format!("Error of saving image: {}", err))
    }
}

impl From<SerdeError> for GenerationsErrors {
    fn from(err: SerdeError) -> Self {
        GenerationsErrors::DeserializeError(format!("Deserialize Error: {}", err))
    }
}
