// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::error::SfResult;

pub mod jwt;

pub enum AuthorizationServer {
    Live,
    Test,
}

impl Display for AuthorizationServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live => write!(f, "https://login.salesforce.com"),
            Self::Test => write!(f, "https://test.salesforce.com"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SfAccessToken {
    pub access_token: String,
    pub scope: String,
    pub instance_url: String,
    pub id: String,
    pub token_type: String,
}

#[async_trait]
pub trait Authenticator {
    async fn get_token(&self) -> SfResult<SfAccessToken>;
}
