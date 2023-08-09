// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use async_trait::async_trait;

use crate::{Authenticator, error::SfResult};

use super::SfAccessToken;

pub struct SessionAuthenticator {
    access_token: String,
    instance_url: String,
}

impl SessionAuthenticator {
    pub fn new(access_token: String, instance_url: String) -> Self {
        Self {
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
            token_type: String::new()
        })
    }
}