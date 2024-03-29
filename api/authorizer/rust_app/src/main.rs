#![allow(dead_code)]

use aws_sdk_ssm;
use envmnt;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use log::info;
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

mod policy_builder;
use policy_builder::{APIGatewayCustomAuthorizerPolicy, PolicyBuilder};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct APIGatewayCustomAuthorizerRequest {
    #[serde(rename = "type")]
    _type: String,
    authorization_token: String,
    method_arn: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct APIGatewayCustomAuthorizerResponse {
    principal_id: String,
    policy_document: APIGatewayCustomAuthorizerPolicy,
    context: serde_json::Value,
}

async fn function_handler(
    event: LambdaEvent<APIGatewayCustomAuthorizerRequest>,
) -> Result<APIGatewayCustomAuthorizerResponse, Error> {
    info!("Event: {:?}", event);

    // Make a get request to https://oauth2.googleapis.com/tokeninfo?id_token={token}
    // to validate the token. This should return a GoogleAuthResponse struct.
    let client = reqwest::Client::new();
    let res = client
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", event.payload.authorization_token)])
        .send()
        .await?
        .json::<GoogleAuthResponse>()
        .await?;

    info!("Received Token Info: {:?}", res);

    let method_arn_array: Vec<&str> = event.payload.method_arn.split(":").collect();
    let api_gateway_arn_tmp: Vec<&str> = method_arn_array[5].split("/").collect();

    let policy_builder = PolicyBuilder::new(
        method_arn_array[3],
        method_arn_array[4],
        api_gateway_arn_tmp[0],
        api_gateway_arn_tmp[1],
    );

    info!("Created policy");

    // Make sure the aud is retrieved from SSM
    // and the iss is retreived as an ENV VAR
    let config = ::aws_config::load_from_env().await;
    let client = aws_sdk_ssm::Client::new(&config);
    let response = client
        .get_parameter()
        .set_name(Some(envmnt::get_or_panic("GoogleAud")))
        .with_decryption(true)
        .send()
        .await?;

    info!("Retrieved parameter response");

    let google_audience = response
        .parameter()
        .expect("Could not unwarp SSM resposne.")
        .value()
        .expect("Value could not be retrieved.");

    if res.aud == google_audience && res.iss == envmnt::get_or_panic("GoogleIss") {
        let response = APIGatewayCustomAuthorizerResponse {
            principal_id: res.sub,
            policy_document: policy_builder.allow_all_methods().build(),
            context: json!({
                "email": res.email,
                "email_verified": res.email_verified,
                "name": res.name,
                "picture": res.picture,
                "given_name": res.given_name,
                "family_name": res.family_name,
            }),
        };

        info!("Positive response being sent {:#?}", response);
        return Ok(response);
    }

    let response = APIGatewayCustomAuthorizerResponse {
        principal_id: "".to_string(),
        policy_document: policy_builder.deny_all_methods().build(),
        context: json!({}),
    };
    info!("Negative response being sent {:#?}", response);
    return Ok(response);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    simple_logger::init_with_level(log::Level::Info)?;
    run(service_fn(function_handler)).await
}

// Create a struct for the Google auth response
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct GoogleAuthResponse {
    iss: String,
    nbf: String,
    aud: String,
    sub: String,
    email: String,
    email_verified: String,
    azp: String,
    name: String,
    picture: String,
    given_name: String,
    family_name: String,
    iat: String,
    exp: String,
    jti: String,
    alg: String,
    kid: String,
    typ: String,
}
