use lambda_runtime::{handler_fn, Context, Error};
use rusoto_ses::Ses;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let (subject, to_addr) = match env::var("TEST").unwrap().as_str() {
        "TRUE" => ("test".to_string(), "seelerei0130@gmail.com".to_string()),
        "FALSE" => ("start_working".to_string(), env::var("TO_ADDR").unwrap()),
        _ => ("test".to_string(), "seelerei0130@gmail.com".to_string()),
    };

    let content_title = event["queryStringParameters"]["content"]
        .as_str()
        .unwrap_or("test");
    let mail_body = match content_title {
        "start_content" => env::var("START_CONTENT1").unwrap(),
        "over_content" => env::var("OVER_CONTENT").unwrap(),
        _ => "test".to_string(),
    };
    let ses_client = rusoto_ses::SesClient::new(rusoto_signature::Region::UsEast1);
    let request = rusoto_ses::SendEmailRequest {
        destination: rusoto_ses::Destination {
            to_addresses: Some(vec![to_addr]),
            ..Default::default()
        },
        message: rusoto_ses::Message {
            body: rusoto_ses::Body {
                text: Some(rusoto_ses::Content {
                    data: mail_body,
                    ..Default::default()
                }),
                ..Default::default()
            },
            subject: rusoto_ses::Content {
                data: subject,
                ..Default::default()
            },
        },
        source: "amon.yamamoto@gmail.com".to_string(),
        ..Default::default()
    };
    match ses_client.send_email(request).await {
        Ok(send_mail_response) => send_mail_response,
        Err(err) => return Err(Box::new(err)),
    };

    Ok(json!({ "statusCode":200,"response": "Ok!" }))
}
