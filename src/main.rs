extern crate clap;
use chrono::prelude::*;
use clap::{App, Arg};
use csv::WriterBuilder;
use itertools::Itertools;
// use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::{thread, time};

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
    time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglData {
    id: u64,
    title: TogglDataTitle,
    items: Vec<TogglItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglResponse {
    data: Vec<TogglData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglDetail {
    description: String,
    start: DateTime<FixedOffset>,
    dur: u64,
    user: String,
    project: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglDetailResponse {
    total_count: u64,
    per_page: u64,
    data: Vec<TogglDetail>,
}

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

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
struct RecordKey {
    user: String,
    project: Option<String>,
    date: NaiveDate,
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

    /*
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
    let user_ids = res.data.iter().map(|d| d.id).collect::<Vec<i64>>();
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

    let project_time_by_user = res
        .data
        .iter()
        .map(|d| {
            let project_times = d
                .items
                .iter()
                .map(|ditem| (&ditem.title.project, ditem.time))
                .collect::<Vec<_>>();
            (&d.title.user, project_times)
        })
        .collect::<HashMap<_, _>>();

    println!("users = {:#?}", users);
    println!("user_ids = {:#?}", user_ids);
    println!("projects = {:#?}", projects);
    println!("project_time_by_user = {:#?}", project_time_by_user);
    */

    let toggl_url = "https://api.track.toggl.com/reports/api/v2/details";
    let client = reqwest::Client::new();
    let first_res = client
        .get(toggl_url)
        .basic_auth(toggl_token, Some("api_token"))
        .query(&[
            ("workspace_id", workspace),
            ("since", date_from),
            ("until", date_to),
            ("user_agent", toggl_email),
        ])
        .send()
        .await?
        .json::<TogglDetailResponse>()
        .await?;
    let max_page = (first_res.total_count as f64 / first_res.per_page as f64).ceil() as u64;

    let mut buf: Vec<TogglDetailResponse> = Vec::new();
    buf.push(first_res);
    for page in 2..=max_page {
        // avoid too many accesses
        thread::sleep(time::Duration::from_millis(2000));
        let tmp_res = client
            .get(toggl_url)
            .basic_auth(toggl_token, Some("api_token"))
            .query(&[
                ("workspace_id", workspace),
                ("since", date_from),
                ("until", date_to),
                ("user_agent", toggl_email),
                ("page", &page.to_string()),
            ])
            .send()
            .await?
            .json::<TogglDetailResponse>()
            .await?;
        buf.push(tmp_res);
    }
    let data: Vec<&TogglDetail> = buf.iter().flat_map(|res| &res.data).collect();

    let dur_time_by_project_user_date: Vec<(RecordKey, u64)> = data
        .iter()
        .map(|d| {
            let u = d.user.clone();
            let p = d.project.clone();
            let start = d.start.naive_local().date();
            let dur = d.dur;
            // ((u, p, start), dur)
            let record_key = RecordKey {
                user: u,
                project: p,
                date: start,
            };
            (record_key, dur)
        })
        .collect();
    println!("{:?}", dur_time_by_project_user_date);

    // sum up duration time by RecordKey
    let summed_dur_time_by_project_user_date: HashMap<RecordKey, u64> =
        dur_time_by_project_user_date
            .into_iter()
            .into_group_map()
            .into_iter()
            .map(|(k, v)| (k, v.iter().sum::<u64>()))
            .collect();
    println!("{:#?}", summed_dur_time_by_project_user_date);

    let start_date: NaiveDate = NaiveDate::parse_from_str(date_from, "%Y-%m-%d").unwrap();
    let end_date: NaiveDate = NaiveDate::parse_from_str(date_to, "%Y-%m-%d").unwrap();

    let csv = create_text_for_csv(
        &summed_dur_time_by_project_user_date,
        &start_date,
        &end_date,
    );
    print!("{}", csv);

    /*
    // create message
    let message = project_time_by_user
        .iter()
        .fold(String::from(""), |acc, item| {
            acc + &format!(
                "*{name}*\n\n```{project_times_text}```",
                name = item.0,
                project_times_text = convert_project_times(item.1) + "\n"
            )
        });
    println!("message = {:#?}", message);

    // send message to Slack
    let mut slack_header = HeaderMap::new();
    let slack_auth_value = format!("Bearer {}", slack_token);
    slack_header.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&slack_auth_value).unwrap(),
    );
    let slack_url = "https://slack.com/api/chat.postMessage";
    let slack_message = SlackMessage {
        channel: String::from(slack_channel),
        text: message,
    };
    let res = client
        .post(slack_url)
        .json(&slack_message)
        .headers(slack_header)
        .send()
        .await?
        .json::<SlackResponse>()
        .await?;
    match res.ok {
        true => println!("Success"),
        false => println!("Error: {:#?}", res.error),
    }
    */
    Ok(())
}

fn convert_project_times(project_times: &Vec<(&Option<String>, i64)>) -> String {
    project_times.iter().fold(String::from(""), |acc, i| {
        acc + &format!(
            "{project: <100}{time: >20}\n",
            project = i.0.clone().unwrap_or("".to_string()),
            time = i.1
        )
    })
}

/// Returns all dates between `start_date` and `end_date`
///
/// The dates are sorted ascendingly.
fn sorted_dates_in_period(start_date: &NaiveDate, end_date: &NaiveDate) -> Vec<NaiveDate> {
    if start_date > end_date {
        panic!("Wrong input")
    };

    let dates: Vec<NaiveDate> = start_date
        .iter_days()
        .take_while(|x| x <= &end_date)
        .collect();
    dates
}

/// Returns report text csv-formatted
///
/// e.g.
/// project,   user, date1, date2, date3, ...
/// projectA, Alice,  0.5,     1,     0, ...
/// projectA,   Bob,    0,     5,     2, ...
fn create_text_for_csv(
    summed_dur_time_by_project_user_date: &HashMap<RecordKey, u64>,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> String {
    let dates = sorted_dates_in_period(start_date, end_date);
    let dates_str: Vec<String> = dates
        .iter()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();

    let projects: HashSet<Option<String>> = summed_dur_time_by_project_user_date
        .iter()
        .map(|(k, _)| k.project.clone())
        .collect();
    let users: HashSet<String> = summed_dur_time_by_project_user_date
        .iter()
        .map(|(k, _)| k.user.clone())
        .collect();

    let mut wtr = WriterBuilder::new().from_writer(vec![]);

    // write header
    let header: Vec<String> = [
        vec!["Project".to_string(), "User".to_string()],
        dates_str.clone(),
    ]
    .concat();
    wtr.write_record(header);
    for p in projects.iter() {
        for u in users.iter() {
            // let row: Vec<String> = vec![p.expect("NoneProject"), u].iter().chain(dates_str.iter()).collect();
            // let row: Vec<String> = [vec![p.clone().unwrap_or(String::from("NoneProject")), u.clone()], dates_str.clone()].concat();
            let durations: Vec<String> = dates
                .iter()
                .map(|d| {
                    let key = &RecordKey {
                        user: u.clone(),
                        project: p.clone(),
                        date: *d,
                    };
                    format_duration_time(
                        summed_dur_time_by_project_user_date.get(key).unwrap_or(&0),
                    )
                })
                .collect();
            let row: Vec<String> = [
                vec![p.clone().unwrap_or("NoneProject".to_string()), u.clone()],
                durations,
            ]
            .concat();
            wtr.write_record(row);
        }
    }
    let data = String::from_utf8(wtr.into_inner().unwrap_or(vec![])).unwrap_or(String::from(""));
    data
}

fn format_duration_time(msec: &u64) -> String {
    let minutes = msec / (1000 * 60);
    let hours = minutes / 60;
    let remainder = minutes % 60;
    let val: f64 = if remainder >= 30 {
        hours as f64 + 0.5
    } else {
        hours as f64
    };
    val.to_string()
}
