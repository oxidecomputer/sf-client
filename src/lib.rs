// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

use error::SfResult;
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use util::deser_body;

pub mod authenticator;
pub use authenticator::{
    jwt::{JwtAuthenticator, LoginClaims},
    Authenticator, AuthorizationServer,
};

use crate::util::is_unit;
pub mod error;
mod util;

pub struct SfClient {
    inner: Client,
    instance_url: String,
    version: String,
    bearer: String,
}

impl SfClient {
    pub async fn new(version: String, authenticator: impl Authenticator) -> SfResult<Self> {
        let token = authenticator.get_token().await?;
        Ok(Self {
            inner: Client::new(),
            instance_url: token.instance_url,
            version,
            bearer: token.access_token,
        })
    }

    fn url(&self, path: &str) -> String {
        let url = format!(
            "{}/services/data/v{}/sobjects/{}",
            self.instance_url, self.version, path
        );
        url
    }

    async fn get<T>(&self, path: &str) -> SfResult<SfResponse<T>>
    where
        T: DeserializeOwned,
    {
        let response = self
            .inner
            .get(&self.url(path))
            .bearer_auth(&self.bearer)
            .send()
            .await?;
        let headers = response.headers().clone();
        let status = response.status().clone();
        let body = response.text().await?;

        match status {
            StatusCode::OK => Ok(SfResponse {
                headers,
                status,
                body: deser_body(&body)?,
            }),
            _ => Err(SfResponse {
                headers,
                status,
                body: Some(deser_body::<Vec<SfApiError>>(&body)?),
            })?,
        }
    }

    async fn post<T>(&self, path: &str, body: T) -> SfResult<SfResponse<CreateObjectResponse>>
    where
        T: Serialize,
    {
        let response = self
            .inner
            .post(&self.url(path))
            .bearer_auth(&self.bearer)
            .json(&body)
            .send()
            .await?;
        let headers = response.headers().clone();
        let status = response.status().clone();
        let body = response.text().await?;

        match status {
            StatusCode::CREATED => Ok(SfResponse {
                headers,
                status,
                body: deser_body(&body)?,
            }),
            _ => Err(SfResponse {
                headers,
                status,
                body: Some(deser_body::<Vec<SfApiError>>(&body)?),
            })?,
        }
    }

    async fn patch<T, U>(&self, path: &str, body: T) -> SfResult<SfResponse<U>>
    where
        T: Serialize,
        U: DeserializeOwned + 'static,
    {
        let response = self
            .inner
            .patch(&self.url(path))
            .bearer_auth(&self.bearer)
            .json(&body)
            .send()
            .await?;
        let headers = response.headers().clone();
        let status = response.status().clone();
        let body = response.text().await?;

        match status {
            StatusCode::NO_CONTENT | StatusCode::CREATED | StatusCode::OK => Ok(SfResponse {
                headers,
                status,
                body: if is_unit::<U>() && body == "" {
                    None
                } else {
                    Some(deser_body(&body)?)
                },
            }),
            _ => Err(SfResponse {
                headers,
                status,
                body: Some(deser_body::<Vec<SfApiError>>(&body)?),
            })?,
        }
    }

    async fn delete(&self, path: &str) -> SfResult<SfResponse<()>> {
        let response = self
            .inner
            .delete(&self.url(path))
            .bearer_auth(&self.bearer)
            .send()
            .await?;
        let headers = response.headers().clone();
        let status = response.status().clone();
        let body = response.text().await?;

        match status {
            StatusCode::NO_CONTENT => Ok(SfResponse {
                headers,
                status,
                body: Some(()),
            }),
            _ => Err(SfResponse {
                headers,
                status,
                body: Some(deser_body::<Vec<SfApiError>>(&body)?),
            })?,
        }
    }

    pub async fn describe_objects(&self) -> SfResult<SfResponse<ObjectDescriptionsResponse>> {
        self.get("").await
    }

    pub async fn describe_object(
        &self,
        object: &str,
    ) -> SfResult<SfResponse<ObjectDescriptionResponse>> {
        self.get(&format!("{}", object)).await
    }

    pub async fn create_object<T>(
        &self,
        object: &str,
        body: T,
    ) -> SfResult<SfResponse<CreateObjectResponse>>
    where
        T: Serialize,
    {
        self.post(&format!("{}", object), body).await
    }

    pub async fn get_object<T>(&self, object: &str, id: &str) -> SfResult<SfResponse<T>>
    where
        T: DeserializeOwned,
    {
        self.get::<T>(&format!("{}/{}", object, id)).await
    }

    pub async fn query<T>(&self, query: &str) -> SfResult<SfResponse<QueryResponse<T>>>
    where
        T: DeserializeOwned,
    {
        let query = urlencoding::encode(query);
        self.get::<QueryResponse<T>>(&format!("query/?q={}", query))
            .await
    }

    pub async fn update_object<T>(
        &self,
        object: &str,
        id: &str,
        body: T,
    ) -> SfResult<SfResponse<()>>
    where
        T: Serialize,
    {
        self.patch(&format!("{}/{}", object, id), body).await
    }

    pub async fn upsert_object<T>(
        &self,
        object: &str,
        id: &ExternalId,
        body: T,
    ) -> SfResult<SfResponse<CreateObjectResponse>>
    where
        T: Serialize,
    {
        self.patch(&format!("{}/{}/{}", object, id.field, id.value), body)
            .await
    }

    pub async fn delete_object(&self, object: &str, id: &str) -> SfResult<SfResponse<()>> {
        self.delete(&format!("{}/{}", object, id)).await
    }
}

#[derive(Debug, Error)]
pub struct SfResponse<T> {
    pub headers: HeaderMap,
    pub status: StatusCode,
    pub body: Option<T>,
}

impl<T> fmt::Display for SfResponse<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Received response with {} status", self.status)
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SfApiError {
    #[serde(rename = "errorCode")]
    pub error_code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QueryResponse<T> {
    #[serde(rename = "totalSize")]
    pub total_size: i32,
    pub done: bool,
    #[serde(rename = "nextRecordsUrl")]
    pub next_records_url: String,
    pub records: Vec<QueryRecord<T>>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QueryRecord<T> {
    pub attributes: QueryRecordAttributes,
    #[serde(flatten)]
    pub object: T,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct QueryRecordAttributes {
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ObjectDescriptionsResponse {
    pub encoding: String,
    #[serde(rename = "maxBatchSize")]
    pub max_batch_size: u32,
    pub sobjects: Vec<ObjectDescription>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ObjectDescriptionResponse {
    #[serde(rename = "objectDescribe")]
    pub object_describe: ObjectDescription,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ObjectDescription {
    pub name: String,
    pub label: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateObjectResponse {
    pub id: Option<String>,
    pub errors: Vec<SfApiError>,
    pub success: bool,
}

pub struct ExternalId {
    pub field: String,
    pub value: String,
}

impl ExternalId {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        authenticator::{
            jwt::{
                tests::{add_token_mock, private_key},
                JwtAuthenticator, LoginClaims,
            },
            AuthorizationServer,
        },
        error::Error,
    };

    use super::*;

    async fn get_client(server: &MockServer) -> SfClient {
        let key = private_key();
        let client_id = "123";
        let aud = AuthorizationServer::Test;
        let sub = "test@company.com";

        let claims = LoginClaims::new(client_id.to_string(), aud, sub.to_string());

        let authenticator = JwtAuthenticator::new(&server.uri(), claims, key);

        let client = SfClient::new("12345.0".to_string(), authenticator)
            .await
            .unwrap();
        client
    }

    #[tokio::test]
    async fn test_describe_object() {
        let server = MockServer::start().await;
        add_token_mock(&server).await;

        let expected_response = ObjectDescriptionResponse {
            object_describe: ObjectDescription {
                name: "Lead".to_string(),
                label: "Lead".to_string(),
            },
        };
        Mock::given(method("GET"))
            .and(path("/services/data/v12345.0/sobjects/Lead"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
            .mount(&server)
            .await;

        let client = get_client(&server).await;
        let response = client.describe_object("Lead").await;

        assert_eq!(expected_response, response.unwrap().body.unwrap());
    }

    #[tokio::test]
    async fn test_describe_objects() {
        let server = MockServer::start().await;
        add_token_mock(&server).await;

        let expected_response = ObjectDescriptionsResponse {
            encoding: "None".to_string(),
            max_batch_size: 200,
            sobjects: vec![ObjectDescription {
                name: "Lead".to_string(),
                label: "Lead".to_string(),
            }],
        };
        Mock::given(method("GET"))
            .and(path("/services/data/v12345.0/sobjects/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
            .mount(&server)
            .await;

        let client = get_client(&server).await;
        let response = client.describe_objects().await;

        assert_eq!(expected_response, response.unwrap().body.unwrap());
    }

    #[tokio::test]
    async fn test_create_object_ok() {
        let server = MockServer::start().await;
        add_token_mock(&server).await;

        let expected_response = CreateObjectResponse {
            id: Some("12345".to_string()),
            errors: vec![],
            success: true,
        };
        Mock::given(method("POST"))
            .and(path("/services/data/v12345.0/sobjects/Lead"))
            .respond_with(ResponseTemplate::new(201).set_body_json(&expected_response))
            .mount(&server)
            .await;

        let client = get_client(&server).await;

        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Lead {
            name: String,
            custom_value: String,
        }
        let response = client
            .create_object(
                "Lead",
                &Lead {
                    name: "Test Lead".to_string(),
                    custom_value: "Non standard value".to_string(),
                },
            )
            .await;

        assert_eq!(expected_response, response.unwrap().body.unwrap());
    }

    #[tokio::test]
    async fn test_create_object_err() {
        let server = MockServer::start().await;
        add_token_mock(&server).await;

        let expected_response = vec![SfApiError {
            error_code: "INVALID_NAME".to_string(),
            message: "Name contains invalid characters".to_string(),
        }];
        Mock::given(method("POST"))
            .and(path("/services/data/v12345.0/sobjects/Lead"))
            .respond_with(ResponseTemplate::new(400).set_body_json(&expected_response))
            .mount(&server)
            .await;

        let client = get_client(&server).await;

        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Lead {
            name: String,
            custom_value: String,
        }
        let response = client
            .create_object(
                "Lead",
                &Lead {
                    name: "Test Lead".to_string(),
                    custom_value: "Non standard value".to_string(),
                },
            )
            .await;

        let err = response.unwrap_err();

        assert!(matches!(err, Error::ApiFailure(_)));

        if let Error::ApiFailure(err) = err {
            assert_eq!(expected_response, err.body.unwrap());
        }
    }

    #[tokio::test]
    async fn test_returns_body_on_deser_failure() {
        let server = MockServer::start().await;
        add_token_mock(&server).await;

        let expected_body = r#"{"invalid":"notvalid"}"#.to_string();

        Mock::given(method("GET"))
            .and(path("/services/data/v12345.0/sobjects/Lead/123"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&expected_body))
            .mount(&server)
            .await;

        let client = get_client(&server).await;

        #[derive(Debug, Deserialize)]
        struct Lead {
            #[allow(dead_code)]
            id: String,
        }

        let response = client.get_object::<Lead>("Lead", "123").await;

        let err = response.unwrap_err();

        assert!(matches!(err, Error::UnexpectedBody { .. }));

        if let Error::UnexpectedBody { body, .. } = err {
            assert_eq!(expected_body, body);
        }
    }
}
