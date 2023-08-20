use aws_config::SdkConfig;
use aws_sdk_sesv2::{
  error::SdkError,
  operation::send_email::{SendEmailError, SendEmailOutput},
  types::{Body, Content, Destination, EmailContent, Message},
};
use aws_smithy_http::body::SdkBody;
use http::Response;
use tokio::sync::OnceCell;

static AWS_CONFIG: OnceCell<SdkConfig> = OnceCell::const_new();
static SES_CLIENT: OnceCell<aws_sdk_sesv2::Client> = OnceCell::const_new();

async fn get_aws_config() -> &'static SdkConfig {
  AWS_CONFIG.get_or_init(|| aws_config::load_from_env()).await
}

async fn get_ses_client() -> &'static aws_sdk_sesv2::Client {
  SES_CLIENT
    .get_or_init(|| async {
      let config = get_aws_config().await;
      aws_sdk_sesv2::Client::new(config)
    })
    .await
}

pub async fn send_email(
  from_address: &str,
  destination: Destination,
  subject: &str,
  body_html: Option<&str>,
  body_text: Option<&str>,
) -> Result<SendEmailOutput, SdkError<SendEmailError, Response<SdkBody>>> {
  let client = get_ses_client().await;
  let mut body = Body::builder();
  if let Some(body_html) = body_html {
    body = body.html(Content::builder().data(body_html).build());
  }
  if let Some(body_text) = body_text {
    body = body.text(Content::builder().data(body_text).build());
  }

  let content = EmailContent::builder().simple(
    Message::builder()
      .subject(Content::builder().data(subject).build())
      .body(body.build())
      .build(),
  );

  client
    .send_email()
    .from_email_address(from_address)
    .destination(destination)
    .content(content.build())
    .send()
    .await
}
