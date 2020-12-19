use crate::values::{Duration, Project, ProjectRecords, User};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{thread, time};

#[derive(Deserialize, Debug)]
struct TogglSummaryResponse {
    data: Vec<TogglSummary>,
}

#[derive(Deserialize, Debug)]
struct TogglSummary {
    id: u64,
    title: TogglSummaryTitle,
    items: Vec<TogglItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TogglSummaryTitle {
    user: User,
}

#[derive(Deserialize, Debug)]
struct TogglItem {
    title: TogglItemTitle,
    time: Duration,
}

#[derive(Deserialize, Debug)]
struct TogglItemTitle {
    project: Project,
}

#[derive(Deserialize, Debug)]
struct TogglDetailResponse {
    total_count: u64,
    per_page: u64,
    data: Vec<TogglDetail>,
}

#[derive(Deserialize, Debug)]
struct TogglDetail {
    description: String,
    start: DateTime<FixedOffset>,
    dur: Duration,
    user: User,
    project: Project,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct RecordKey {
    pub user: User,
    pub project: Project,
    pub date: NaiveDate,
}

pub struct TogglAccessor {
    pub token: String,
    pub workspace: String,
    pub email: String,
}

impl TogglAccessor {
    const SUMMARY_REPORT_URL: &'static str = "https://api.track.toggl.com/reports/api/v2/summary";
    const DETAILED_REPORT_URL: &'static str = "https://api.track.toggl.com/reports/api/v2/details";

    /// Fetches summary report from Toggl API and convert it to HashMap
    pub async fn fetch_summary_report(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<ProjectRecords, Box<dyn std::error::Error>> {
        let res = self.fetch_summary(date_from, date_to).await?;
        let project_time_by_user = Self::convert_summary_to_hashmap(&res);
        Ok(project_time_by_user)
    }

    /// Fetches summary report from Toggl API
    async fn fetch_summary(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<TogglSummaryResponse, Box<dyn std::error::Error>> {
        let url = Self::SUMMARY_REPORT_URL;
        let client = reqwest::Client::new();
        let res: TogglSummaryResponse = client
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
            .json::<TogglSummaryResponse>()
            .await?;
        Ok(res)
    }

    /// Converts summary report fetched from Toggl API to HashMap
    fn convert_summary_to_hashmap(res: &TogglSummaryResponse) -> ProjectRecords {
        let records = res
            .data
            .iter()
            .map(|d| {
                let project_times = d
                    .items
                    .iter()
                    .map(|ditem| (ditem.title.project.clone(), ditem.time))
                    .collect();
                (d.title.user.clone(), project_times)
            })
            .collect();
        ProjectRecords::new(records)
    }

    /// Fetches detailed report from Toggl API and convert it to Vec
    pub async fn fetch_detailed_report(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<(RecordKey, Duration)>, Box<dyn std::error::Error>> {
        let details = self.fetch_details(date_from, date_to).await?;
        let dur_time_by_project_user_date: Vec<(RecordKey, Duration)> =
            Self::convert_details_to_vec(&details);
        Ok(dur_time_by_project_user_date)
    }

    /// Fetches detailed report from Toggl API
    ///
    /// TODO: rewrite paging process
    async fn fetch_details(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<TogglDetail>, Box<dyn std::error::Error>> {
        let url = Self::DETAILED_REPORT_URL;
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
            // to avoid rapid accesses
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
        let details: Vec<TogglDetail> = buf.into_iter().flat_map(|res| res.data).collect();
        Ok(details)
    }

    /// Converts detailed report fetched from Toggl API to Vec
    fn convert_details_to_vec(data: &Vec<TogglDetail>) -> Vec<(RecordKey, Duration)> {
        data.iter()
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
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convert_summary_to_hashmap_must_work_well() {
        let user_name_1 = User::new("Alice");
        let user_name_2 = User::new("Bob");
        let project_1 = Project::new(Some("ProjectA"));
        let project_2 = Project::new(Some("ProjectB"));
        let project_3 = Project::new(None);
        let t1 = Duration::new(100);
        let t2 = Duration::new(200);
        let t3 = Duration::new(400);
        let t4 = Duration::new(800);

        let res = TogglSummaryResponse {
            data: vec![
                TogglSummary {
                    id: 1,
                    title: TogglSummaryTitle {
                        user: user_name_1.clone(),
                    },
                    items: vec![
                        TogglItem {
                            title: TogglItemTitle {
                                project: project_1.clone(),
                            },
                            time: t1,
                        },
                        TogglItem {
                            title: TogglItemTitle {
                                project: project_2.clone(),
                            },
                            time: t2,
                        },
                    ],
                },
                TogglSummary {
                    id: 2,
                    title: TogglSummaryTitle {
                        user: user_name_2.clone(),
                    },
                    items: vec![
                        TogglItem {
                            title: TogglItemTitle {
                                project: project_3.clone(),
                            },
                            time: t3,
                        },
                        TogglItem {
                            title: TogglItemTitle {
                                project: project_1.clone(),
                            },
                            time: t4,
                        },
                    ],
                },
            ],
        };
        let actual = TogglAccessor::convert_summary_to_hashmap(&res);
        let expected: ProjectRecords = ProjectRecords::new(vec![
            (
                user_name_1.clone(),
                vec![(project_1.clone(), t1), (project_2.clone(), t2)],
            ),
            (
                user_name_2.clone(),
                vec![(project_3.clone(), t3), (project_1.clone(), t4)],
            ),
        ]
        .iter()
        .cloned()
        .collect());
        assert_eq!(actual, expected)
    }

    #[test]
    fn convert_details_to_vec_must_work() {
        let user1 = User::new("Alice");
        let user2 = User::new("Bob");
        let project = Project::new(Some("ProjectA"));
        let dur1 = Duration::new(100);
        let dur2 = Duration::new(200);
        let dur3 = Duration::new(400);
        let desc = "".to_string();

        let data = vec![
            TogglDetail {
                description: desc.clone(),
                start: "2020-12-01T10:00:00+09:00"
                    .parse::<DateTime<FixedOffset>>()
                    .expect(""),
                dur: dur1,
                user: user1.clone(),
                project: project.clone(),
            },
            TogglDetail {
                description: desc.clone(),
                start: "2020-12-01T20:00:00+09:00"
                    .parse::<DateTime<FixedOffset>>()
                    .expect(""),
                dur: dur2,
                user: user1.clone(),
                project: project.clone(),
            },
            TogglDetail {
                description: desc.clone(),
                start: "2020-12-01T15:00:00+09:00"
                    .parse::<DateTime<FixedOffset>>()
                    .expect(""),
                dur: dur3,
                user: user2.clone(),
                project: project.clone(),
            },
        ];

        let actual = TogglAccessor::convert_details_to_vec(&data);
        let expected = vec![
            (
                RecordKey {
                    user: user1.clone(),
                    project: project.clone(),
                    date: "2020-12-01".parse::<NaiveDate>().expect(""),
                },
                dur1,
            ),
            (
                RecordKey {
                    user: user1.clone(),
                    project: project.clone(),
                    date: "2020-12-01".parse::<NaiveDate>().expect(""),
                },
                dur2,
            ),
            (
                RecordKey {
                    user: user2.clone(),
                    project: project.clone(),
                    date: "2020-12-01".parse::<NaiveDate>().expect(""),
                },
                dur3,
            ),
        ];
        assert_eq!(actual, expected)
    }
}
