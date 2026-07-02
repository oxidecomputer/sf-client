// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use reqwest::Error as ClientError;
use serde::Deserialize;
use std::env::VarError;
use thiserror::Error;

use crate::{SfApiError, SfResponse};

#[derive(Debug, Error)]
pub enum Error {
    #[error("API request failed {0}")]
    ApiFailure(Box<SfResponse<Vec<SfApiError>>>),
    #[error("Request failed {0}")]
    Client(#[from] ClientError),
    #[error("Failed to create authentication assertion {0}")]
    FailedToCreateAssertion(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to load key {0}")]
    LoadKey(#[from] std::io::Error),
    #[error("Login request failed {0}")]
    LoginFailure(Box<SfResponse<SfLoginError>>),
    #[error("Failed to find necessary environment variables {0}")]
    MissingEnvConfig(#[from] VarError),
    #[error("Failed to deserialize response")]
    UnexpectedBody {
        error: serde_json::Error,
        body: String,
    },
    #[error("Unknown request failed {0}")]
    UnknownApiFailure(Box<SfResponse<String>>),
}

pub type SfResult<T> = Result<T, Error>;

impl From<SfResponse<Vec<SfApiError>>> for Error {
    fn from(response: SfResponse<Vec<SfApiError>>) -> Self {
        Self::ApiFailure(Box::new(response))
    }
}

impl From<SfResponse<SfLoginError>> for Error {
    fn from(response: SfResponse<SfLoginError>) -> Self {
        Self::LoginFailure(Box::new(response))
    }
}

impl From<SfResponse<String>> for Error {
    fn from(response: SfResponse<String>) -> Self {
        Self::UnknownApiFailure(Box::new(response))
    }
}

#[derive(Debug, Deserialize)]
pub struct SfLoginError {
    pub error: String,
    pub error_description: String,
}
