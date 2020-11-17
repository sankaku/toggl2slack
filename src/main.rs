extern crate clap;
use toggl2slack::message;
use toggl2slack::slack;
use toggl2slack::toggl;

use chrono::prelude::*;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("toggl2slack")
        .version("1.0")
        .author("sankaku <sankaku.git@gmail.com>")
        .about("Fetch toggl report and send it to Slack")
        .arg(
            Arg::new("toggl_token")
                .short('t')
                .long("toggl_token")
                .value_name("TOGGL_API_TOKEN")
                .about("Sets API token for toggl")
                .takes_value(true),
        )
        .arg(
            Arg::new("workspace")
                .long("workspace")
                .value_name("TOGGL_WORKSPACE")
                .about("Sets workspace id for toggl")
                .takes_value(true),
        )
        .arg(
            Arg::new("toggl_email")
                .long("toggl_email")
                .value_name("TOGGL_MAIL_ADDRESS")
                .about("Sets email address for toggl")
                .takes_value(true),
        )
        .arg(
            Arg::new("date_from")
                .long("date_from")
                .value_name("DATE_FROM")
                .about("Sets the start date of report period. eg. 2020-01-01")
                .required(true),
        )
        .arg(
            Arg::new("date_to")
                .long("date_to")
                .value_name("DATE_TO")
                .about("Sets the end date of report period. eg. 2020-01-31")
                .required(true),
        )
        .arg(
            Arg::new("slack_token")
                .long("slack_token")
                .value_name("Slack token")
                .about("Sets Slack API token")
                .required(true),
        )
        .arg(
            Arg::new("slack_channel")
                .long("slack_channel")
                .value_name("Slack channel")
                .about("Sets Slack channel")
                .required(true),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple(true)
                .about("Sets the level of verbosity"),
        )
        .get_matches();

    let toggl_token = matches.value_of("toggl_token").unwrap_or("");
    println!("Value for toggl_token: {}", toggl_token);
    let workspace = matches.value_of("workspace").unwrap_or("");
    println!("Value for workspace: {}", workspace);
    let toggl_email = matches.value_of("toggl_email").unwrap_or("");
    println!("Value for toggl_email: {}", toggl_email);
    let date_from = matches.value_of("date_from").unwrap_or("");
    println!("Value for date_from: {}", date_from);
    let date_to = matches.value_of("date_to").unwrap_or("");
    println!("Value for date_to: {}", date_to);
    let slack_token = matches.value_of("slack_token").unwrap_or("");
    println!("Value for slack_token: {}", slack_token);
    let slack_channel = matches.value_of("slack_channel").unwrap_or("");
    println!("Value for slack_channel: {}", slack_channel);

    let toggl_accessor = toggl::TogglAccessor {
        token: toggl_token.to_string(),
        workspace: workspace.to_string(),
        email: toggl_email.to_string(),
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
    let summary_message = message_creator.convert_project_times(&summary_report);
    let detailed_message =
        message_creator.create_text_for_csv(&detailed_report, &start_date, &end_date);

    print!(
        "[summary_message in {date_from} to {date_to}]\n{summary_message}",
        date_from = &date_from,
        date_to = &date_to,
        summary_message = &summary_message
    );
    print!(
        "[detailed_message in {date_from} to {date_to}]\n{detailed_message}",
        date_from = &date_from,
        date_to = &date_to,
        detailed_message = &detailed_message
    );

    let sender = slack::SlackAccessor {
        token: slack_token.to_string(),
    };
    // sender.send_message(slack_channel, &summary_message).await?;
    // sender.send_message(slack_channel, &detailed_message).await?;

    Ok(())
}
