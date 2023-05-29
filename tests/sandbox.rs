// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2023 Oxide Computer Company

// Tests to be run against a SalesForce Sandbox

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sf_client::{
    authenticator::{
        jwt::{JwtAuthenticator, LoginClaims},
        AuthorizationServer,
    },
    error::Error,
    ExternalId, SfClient,
};
use std::env::var;

fn tvar(name: &str) -> String {
    var(name).expect(&format!("Failed to find expected variable {}", name))
}

#[derive(Debug, Deserialize, Serialize)]
struct Lead {
    #[serde(rename = "FirstName")]
    first_name: String,
    #[serde(rename = "LastName")]
    last_name: String,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Company")]
    company: String,
    #[serde(rename = "NumberOfEmployees")]
    number_of_employees: Option<i64>,
    #[serde(rename = "LeadSource")]
    lead_source: String,
    #[serde(rename = "Interest__c")]
    interest: String,
    #[serde(rename = "External_Service_Record_Id__c")]
    external_service_record_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LeadUpsert {
    #[serde(rename = "FirstName")]
    first_name: String,
    #[serde(rename = "LastName")]
    last_name: String,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Company")]
    company: String,
    #[serde(rename = "NumberOfEmployees")]
    number_of_employees: Option<i64>,
    #[serde(rename = "LeadSource")]
    lead_source: String,
    #[serde(rename = "Interest__c")]
    interest: String,
}

#[ignore]
#[tokio::test]
async fn test_lead_crud() {
    let claims = LoginClaims::new(
        tvar("CLIENT_ID"),
        AuthorizationServer::Test,
        tvar("SUBJECT"),
    );
    let mut authenticator = JwtAuthenticator::new(&tvar("INSTANCE_DOMAIN"), claims, vec![]);
    authenticator.load_rsa_pem("sf_test.key").unwrap();

    let client = SfClient::new(tvar("VERSION"), authenticator)
        .await
        .expect("Failed to create client");

    let object = client
        .create_object(
            "Lead",
            &Lead {
                first_name: "First_1".to_string(),
                last_name: "Last_1".to_string(),
                company: "Test_1".to_string(),
                email: "test_1@test.com".to_string(),
                number_of_employees: Some(123),
                lead_source: "Waitlist".to_string(),
                interest: "Heard on the radio".to_string(),
                external_service_record_id: "external_12345".to_string(),
            },
        )
        .await
        .unwrap()
        .body
        .unwrap()
        .id
        .unwrap();

    client
        .update_object(
            "Lead",
            &object,
            &Lead {
                first_name: "First_2".to_string(),
                last_name: "Last_2".to_string(),
                company: "Test_2".to_string(),
                email: "test_2@test.com".to_string(),
                number_of_employees: Some(456),
                lead_source: "Waitlist_2".to_string(),
                interest: "Heard on the radio 2".to_string(),
                external_service_record_id: "external_12345_2".to_string(),
            },
        )
        .await
        .unwrap();

    let lead = client
        .get_object::<Lead>("Lead", &object)
        .await
        .unwrap()
        .body
        .unwrap();

    assert_eq!("First_2", lead.first_name);
    assert_eq!("Last_2", lead.last_name);
    assert_eq!("Test_2", lead.company);
    assert_eq!("test_2@test.com", lead.email);
    assert_eq!(Some(456), lead.number_of_employees);
    assert_eq!("Waitlist_2", lead.lead_source);
    assert_eq!("Heard on the radio 2", lead.interest);
    assert_eq!("external_12345_2", lead.external_service_record_id);

    client
        .upsert_object(
            "Lead",
            &ExternalId::new(
                "External_Service_Record_Id__c".to_string(),
                "external_12345_2".to_string(),
            ),
            &LeadUpsert {
                first_name: "First_3".to_string(),
                last_name: "Last_3".to_string(),
                company: "Test_3".to_string(),
                email: "test_3@test.com".to_string(),
                number_of_employees: Some(456),
                lead_source: "Waitlist_3".to_string(),
                interest: "Heard on the radio 3".to_string(),
            },
        )
        .await
        .unwrap();

    client.delete_object("Lead", &object).await.unwrap();

    let error = client
        .get_object::<Lead>("Lead", &object)
        .await
        .unwrap_err();

    assert!(matches!(error, Error::ApiFailure(_)));

    // Ensured that this will run by the assertion above
    if let Error::ApiFailure(response) = error {
        assert_eq!(StatusCode::NOT_FOUND, response.status);
    }
}
