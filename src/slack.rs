use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct SlackMessage {
    channel: String,
    text: String,
}

#[derive(Deserialize, Debug)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

pub struct SlackAccessor {
    pub token: String,
}
impl SlackAccessor {
    const URL_POST_MESSAGE: &'static str = "https://slack.com/api/chat.postMessage";

    pub async fn send_message(
        &self,
        channel: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // send message to Slack
        let mut slack_header = HeaderMap::new();
        let slack_auth_value = format!("Bearer {}", &self.token);
        slack_header.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&slack_auth_value).unwrap(),
        );
        let slack_message = SlackMessage {
            channel: channel.to_string(),
            text: message.to_string(),
        };
        let client = reqwest::Client::new();
        let res = client
            .post(Self::URL_POST_MESSAGE)
            .json(&slack_message)
            .headers(slack_header)
            .send()
            .await?
            .json::<SlackResponse>()
            .await?;
        match res.ok {
            true => Ok(println!("Success")),
            false => Ok(println!("Error: {:#?}", res.error)),
        }
    }
}
