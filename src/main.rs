use chrono::prelude::*;
use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, RequestExt, Response,
};
use rusoto_ses::Ses;
use std::env;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let (subject, to_addr, is_test) = match env::var("TEST").unwrap().as_str() {
        "TRUE" => (
            "test".to_string(),
            "seelerei0130@gmail.com".to_string(),
            true,
        ),
        "FALSE" => (
            "start_working".to_string(),
            env::var("TO_ADDR").unwrap(),
            false,
        ),
        _ => (
            "test".to_string(),
            "seelerei0130@gmail.com".to_string(),
            true,
        ),
    };
    let content_title = match event.query_string_parameters().get("content") {
        Some(c) => c.to_string(),
        None => "test".to_string(),
    };
    let mail_body = match content_title.as_str() {
        "start_content" => {
            let now = Local::now();
            if now.hour() <= 14 {
                env::var("START_CONTENT1").unwrap()
            } else {
                return Ok(Response::builder()
                    .status(400)
                    .body("content not match time".into())
                    .expect("failed to render response"));
            }
        }
        "over_content" => {
            let now = Local::now();
            if now.hour() > 14 {
                env::var("OVER_CONTENT").unwrap()
            } else {
                return Ok(Response::builder()
                    .status(400)
                    .body("content not match time".into())
                    .expect("failed to render response"));
            }
        }
        _ => {
            if is_test {
                "test".to_string()
            } else {
                return Ok(Response::builder()
                    .status(400)
                    .body("can not detect content parameter".into())
                    .expect("failed to render response"));
            }
        }
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

    Ok("Ok".to_string().into_response())
}
