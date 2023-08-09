// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{Client as HttpClient, StatusCode};
use serde::Serialize;
use std::{env::var, fs::File, io::Read, ops::Add, path::Path};

use crate::{
    error::{SfLoginError, SfResult},
    SfResponse,
};

use super::{Authenticator, AuthorizationServer, SfAccessToken};

#[derive(Clone, Debug, Serialize)]
pub struct LoginClaims {
    iss: String,
    aud: String,
    sub: String,
    exp: i64,
}

impl LoginClaims {
    pub fn new(iss: String, aud: AuthorizationServer, sub: String) -> Self {
        Self {
            iss,
            aud: format!("{}", aud),
            sub,
            exp: Utc::now().add(Duration::seconds(60)).timestamp(),
        }
    }

    pub fn from_env(aud: AuthorizationServer) -> SfResult<Self> {
        Ok(Self::new(
            var("SALESFORCE_CLIENT_ID")?,
            aud,
            var("SALESFORCE_USER")?,
        ))
    }
}

#[derive(Debug, Serialize)]
struct LoginForm {
    grant_type: String,
    assertion: String,
    format: LoginResponseFormat,
}

#[derive(Debug, Serialize)]
enum LoginResponseFormat {
    #[serde(rename = "json")]
    Json,
    #[allow(dead_code)]
    #[serde(rename = "urlencoded")]
    UrlEncoded,
    #[allow(dead_code)]
    #[serde(rename = "xml")]
    Xml,
}

impl LoginForm {
    pub fn new(claims: &LoginClaims, key: &[u8]) -> SfResult<Self> {
        Ok(Self {
            grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
            assertion: Self::create_assertion(claims, &key)?,
            format: LoginResponseFormat::Json,
        })
    }

    fn header() -> Header {
        Header::new(Algorithm::RS256)
    }

    fn create_assertion(claims: &LoginClaims, key: &[u8]) -> SfResult<String> {
        let enc_key = EncodingKey::from_rsa_pem(&key)?;
        Ok(encode(&Self::header(), claims, &enc_key)?)
    }
}

pub struct JwtAuthenticator {
    inner: HttpClient,
    instance: String,
    key: Vec<u8>,
    claims: LoginClaims,
}

impl JwtAuthenticator {
    pub fn new(instance_domain: &str, claims: LoginClaims, key: Vec<u8>) -> Self {
        Self {
            inner: HttpClient::new(),
            instance: if instance_domain.starts_with("http") {
                instance_domain.trim_end_matches("/").to_string()
            } else {
                format!("https://{}", instance_domain.trim_end_matches("/"))
            },
            key,
            claims,
        }
    }

    pub fn from_env(claims: LoginClaims) -> SfResult<Self> {
        Ok(Self::new(
            &var("SALESFORCE_DOMAIN")?,
            claims,
            var("SALESFORCE_KEY")?.into(),
        ))
    }

    pub fn key(&mut self, key: Vec<u8>) -> &mut Self {
        self.key = key;
        self
    }

    pub fn load_rsa_pem<T>(&mut self, path: T) -> SfResult<&mut Self>
    where
        T: AsRef<Path>,
    {
        let mut f = File::open(path.as_ref())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        self.key = buf;

        Ok(self)
    }
}

#[async_trait]
impl Authenticator for JwtAuthenticator {
    async fn get_token(&self) -> SfResult<SfAccessToken> {
        let form = LoginForm::new(&self.claims, &self.key)?;
        let response = self
            .inner
            .post(&format!("{}/services/oauth2/token", self.instance))
            .form(&form)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(SfResponse {
                headers: response.headers().clone(),
                status: response.status(),
                body: Some(response.json::<SfLoginError>().await?),
            })?,
        }
    }

    async fn user_info(&self) -> SfResult<serde_json::Value> {
        let token = self.get_token().await?;

        let response = self
            .inner
            .get(&format!("{}/services/oauth2/token", self.instance))
            .bearer_auth(token.access_token)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(SfResponse {
                headers: response.headers().clone(),
                status: response.status(),
                body: Some(response.json::<SfLoginError>().await?),
            })?,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;

    pub fn private_key() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        RsaPrivateKey::new(&mut rng, 2048)
            .unwrap()
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    pub async fn add_token_mock(server: &MockServer) -> SfAccessToken {
        let mock_response = SfAccessToken {
            access_token: "access_token".to_string(),
            scope: "scope".to_string(),
            instance_url: server.uri(),
            id: "id".to_string(),
            token_type: "token_type".to_string(),
        };

        Mock::given(method("POST"))
            .and(path("/services/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&server)
            .await;

        mock_response
    }

    #[tokio::test]
    async fn get_token() {
        let mock_server = MockServer::start().await;

        let claims = LoginClaims::new(
            "sf-client-id".to_string(),
            AuthorizationServer::Test,
            "test@company".to_string(),
        );
        let authenticator =
            JwtAuthenticator::new(&mock_server.uri(), claims.clone(), private_key());

        let mock_response = add_token_mock(&mock_server).await;
        let token = authenticator.get_token().await;

        assert_eq!(mock_response, token.unwrap());
    }
}
