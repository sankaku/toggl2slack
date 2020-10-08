extern crate clap;
use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
struct TogglItemTitle {
    project: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglDataTitle {
    user: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglItem {
    title: TogglItemTitle,
    time: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglData {
    id: i64,
    title: TogglDataTitle,
    items: Vec<TogglItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglResponse {
    data: Vec<TogglData>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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

    let toggl_url = "https://api.track.toggl.com/reports/api/v2/summary";
    let client = reqwest::Client::new();
    let res = client
        .get(toggl_url)
        .basic_auth(toggl_token, Some("api_token"))
        .query(&[
            ("workspace_id", workspace),
            ("since", date_from),
            ("until", date_to),
            ("user_agent", toggl_email),
            ("grouping", "users"),
            ("subgrouping", "projects"),
        ])
        .send()
        .await?
        .json::<TogglResponse>()
        .await?;

    let users = res
        .data
        .iter()
        .map(|d| &d.title.user)
        .collect::<Vec<&String>>();
    let user_ids = res.data.iter().map(|d| &d.id).collect::<Vec<&i64>>();
    let projects = res
        .data
        .iter()
        .flat_map(|d| {
            d.items
                .iter()
                .flat_map(|dd| &dd.title.project)
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>();
    println!("users = {:#?}", users);
    println!("user_ids = {:#?}", user_ids);
    println!("projects = {:#?}", projects);

    // println!("{:#?}", res);

    Ok(())
}
