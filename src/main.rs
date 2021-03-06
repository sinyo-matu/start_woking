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
        "FALSE" => (
            "start_working".to_string(),
            env::var("TO_ADDR").unwrap(),
            false,
        ),
        _ => ("test".to_string(), env::var("TEST_ADDR").unwrap(), true),
    };
    let content_title = match event.query_string_parameters().get("content") {
        Some(c) => c.to_string(),
        None => {
            return Ok(Response::builder()
                .status(400)
                .body("can not detect content parameter".into())
                .expect("failed to render response"))
        }
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
                    .body("content parameter not matched".into())
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
        source: env::var("FROM_ADDR").unwrap(),
        ..Default::default()
    };
    if let Err(err) = ses_client.send_email(request).await {
        return Ok(Response::builder()
            .status(400)
            .body(format!("error:{}", err).into())
            .expect("failed to render response"));
    };

    Ok("Ok".to_string().into_response())
}
