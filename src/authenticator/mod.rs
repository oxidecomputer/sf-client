// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::error::SfResult;

pub mod jwt;
pub mod session;

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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SfUserInfo {
    pub sub: String,
    pub user_id: String,
    pub organization_id: String,
    pub preferred_username: String,
    pub nickname: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub given_name: String,
    pub family_name: String,
    pub zoneinfo: String,
    pub profile: String,
    pub picture: String,
    pub phone_number: String,
    pub phone_number_verified: bool,
    pub is_salesforce_integration_user: bool,
    pub active: bool,
    pub user_type: String,
    pub language: String,
    pub locale: String,
    #[serde(rename = "utcOffset")]
    pub utc_offset: i64,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait Authenticator {
    async fn get_token(&self) -> SfResult<SfAccessToken>;
    async fn user_info(&self) -> SfResult<SfUserInfo>;
}
