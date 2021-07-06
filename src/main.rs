extern crate clap;
use toggl2slack::message;
use toggl2slack::slack;
use toggl2slack::toggl;

use chrono::prelude::*;
use clap::{App, Arg};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Env {
    toggl_token: String,
    toggl_workspace: String,
    toggl_email: String,
    slack_token: String,
    slack_channel: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = envy::from_env::<Env>().expect("Invalid environment variables");

    let matches = App::new("toggl2slack")
        .version("1.0")
        .author("sankaku <sankaku.git@gmail.com>")
        .about("Fetch toggl report and send it to Slack")
        .arg(
            Arg::new("date_from")
                .long("date_from")
                .value_name("DATE_FROM")
                .about("Sets the start date of report period(YYYY-MM-DD). eg. 2020-01-01")
                .required(true),
        )
        .arg(
            Arg::new("date_to")
                .long("date_to")
                .value_name("DATE_TO")
                .about("Sets the end date of report period(YYYY-MM-DD). eg. 2020-01-31")
                .required(true),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple(true)
                .about("Sets the level of verbosity"),
        )
        .get_matches();

    let date_from = matches.value_of("date_from").unwrap_or("");
    let date_to = matches.value_of("date_to").unwrap_or("");

    let toggl_accessor = toggl::TogglAccessor {
        token: env.toggl_token.to_string(),
        workspace: env.toggl_workspace.to_string(),
        email: env.toggl_email.to_string(),
    };
    let summary_report = toggl_accessor
        .fetch_summary_report(date_from, date_to)
        .await?;
    let detailed_report = toggl_accessor
        .fetch_detailed_report(date_from, date_to)
        .await?;

    let start_date: NaiveDate = NaiveDate::parse_from_str(date_from, "%Y-%m-%d").unwrap();
    let end_date: NaiveDate = NaiveDate::parse_from_str(date_to, "%Y-%m-%d").unwrap();

    let message_creator = message::MessageCreator {};
    let summary_message =
        message_creator.get_project_message(&summary_report, &start_date, &end_date);
    let detailed_message =
        message_creator.create_text_for_csv(&detailed_report, &start_date, &end_date);

    print_summary_message(&date_from, &date_to, &summary_message);
    print_detailed_message(&date_from, &date_to, &detailed_message);

    let sender = slack::SlackAccessor {
        token: env.slack_token.to_string(),
    };
    // sender.send_message(slack_channel, &summary_message).await?;
    // sender.send_message(slack_channel, &detailed_message).await?;

    Ok(())
}

/// for debug
fn print_summary_message(date_from: &str, date_to: &str, message: &str) -> () {
    println!(
        "[summary_message in {date_from} to {date_to}]\n{summary_message}",
        date_from = &date_from,
        date_to = &date_to,
        summary_message = &message
    )
}

/// for debug
fn print_detailed_message(date_from: &str, date_to: &str, message: &str) -> () {
    println!(
        "[detailed_message in {date_from} to {date_to}]\n{detailed_message}",
        date_from = &date_from,
        date_to = &date_to,
        detailed_message = &message
    )
}
