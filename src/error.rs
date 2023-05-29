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
    ApiFailure(#[from] SfResponse<Vec<SfApiError>>),
    #[error("Request failed {0}")]
    Client(#[from] ClientError),
    #[error("Failed to create authentication assertion {0}")]
    FailedToCreateAssertion(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to load key {0}")]
    LoadKey(#[from] std::io::Error),
    #[error("Login request failed {0}")]
    LoginFailure(#[from] SfResponse<SfLoginError>),
    #[error("Failed to find necessary environment variables {0}")]
    MissingEnvConfig(#[from] VarError),
}

pub type SfResult<T> = Result<T, Error>;

#[derive(Debug, Deserialize)]
pub struct SfLoginError {
    pub error: String,
    pub error_description: String,
}
