use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct RecordKey {
    pub user: String,
    pub project: Option<String>,
    pub date: NaiveDate,
}

pub struct TogglAccessor {
    pub token: String,
    pub workspace: String,
    pub email: String,
}
impl TogglAccessor {
    pub async fn fetch_summary_report(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<HashMap<String, Vec<(Option<String>, u64)>>, Box<dyn std::error::Error>> {
        let url = "https://api.track.toggl.com/reports/api/v2/summary";
        let client = reqwest::Client::new();
        let res: TogglResponse = client
            .get(url)
            .basic_auth(&self.token, Some("api_token"))
            .query(&[
                ("workspace_id", &self.workspace as &str),
                ("since", &date_from),
                ("until", &date_to),
                ("user_agent", &self.email),
                ("grouping", "users"),
                ("subgrouping", "projects"),
            ])
            .send()
            .await?
            .json::<TogglResponse>()
            .await?;

        let project_time_by_user: HashMap<String, Vec<(Option<String>, u64)>> = res
            .data
            .into_iter()
            .map(|d| {
                let project_times = d
                    .items
                    .into_iter()
                    .map(|ditem| (ditem.title.project, ditem.time))
                    .collect::<Vec<_>>();
                (d.title.user, project_times)
            })
            .collect();

        Ok(project_time_by_user)
    }

    pub async fn fetch_detailed_report(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<(RecordKey, u64)>, Box<dyn std::error::Error>> {
        let url = "https://api.track.toggl.com/reports/api/v2/details";
        let client = reqwest::Client::new();
        let first_res = client
            .get(url)
            .basic_auth(&self.token, Some("api_token"))
            .query(&[
                ("workspace_id", &self.workspace as &str),
                ("since", &date_from),
                ("until", &date_to),
                ("user_agent", &self.email),
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
                .get(url)
                .basic_auth(&self.token, Some("api_token"))
                .query(&[
                    ("workspace_id", &self.workspace as &str),
                    ("since", date_from),
                    ("until", date_to),
                    ("user_agent", &self.email),
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
                let record_key = RecordKey {
                    user: u,
                    project: p,
                    date: start,
                };
                (record_key, dur)
            })
            .collect();
        Ok(dur_time_by_project_user_date)
    }
}
