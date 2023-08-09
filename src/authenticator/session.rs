// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use async_trait::async_trait;
use reqwest::{Client as HttpClient, StatusCode};

use crate::{error::{SfResult, SfLoginError}, Authenticator, SfResponse};

use super::{SfAccessToken, SfUserInfo};

pub struct SessionAuthenticator {
    inner: HttpClient,
    access_token: String,
    instance_url: String,
}

impl SessionAuthenticator {
    pub fn new(access_token: String, instance_url: String) -> Self {
        Self {
            inner: HttpClient::new(),
            access_token,
            instance_url,
        }
    }
}

#[async_trait]
impl Authenticator for SessionAuthenticator {
    async fn get_token(&self) -> SfResult<SfAccessToken> {
        Ok(SfAccessToken {
            access_token: self.access_token.clone(),
            scope: String::new(),
            instance_url: self.instance_url.clone(),
            id: String::new(),
            token_type: String::new(),
        })
    }

    async fn user_info(&self) -> SfResult<SfUserInfo> {
        let token = self.get_token().await?;

        let response = self
            .inner
            .get(&format!("{}/services/oauth2/userinfo", self.instance_url))
            .bearer_auth(token.access_token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(SfResponse {
                headers: response.headers().clone(),
                status: response.status(),
                body: Some(response.text().await?),
            })?,
        }
    }
}
