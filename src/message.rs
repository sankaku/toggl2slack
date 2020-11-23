use crate::toggl::RecordKey;
use chrono::prelude::*;
use csv::WriterBuilder;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub struct MessageCreator {}
impl MessageCreator {
    const NONE_PROJECT_LABEL: &'static str = "EmptyProject";

    /// Format project-duration vector to string
    pub fn convert_project_times(
        &self,
        project_times_by_user: &HashMap<String, Vec<(Option<String>, u64)>>,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let title = format!(
            "*Toggl summary report* [{start}-{end}]\n",
            start = start_date.format("%Y/%m/%d"),
            end = end_date.format("%Y/%m/%d")
        );
        project_times_by_user
            .iter()
            .fold(String::from(title), |acc, (u, project_times)| {
                acc + &format!(
                    "\n*{name}*\n\n```{project_times_text}```",
                    name = u,
                    project_times_text =
                        project_times
                            .iter()
                            .fold(String::from(""), |acc, (p, dur)| {
                                acc + &format!(
                                    "{project}: {time}h\n",
                                    project =
                                        p.clone().unwrap_or(Self::NONE_PROJECT_LABEL.to_string()),
                                    time = Self::format_duration_time(dur),
                                )
                            })
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
    pub fn create_text_for_csv(
        &self,
        dur_time_by_project_user_date: &Vec<(RecordKey, u64)>,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let summed_dur_time_by_project_user_date =
            Self::sumup_durations(&dur_time_by_project_user_date);
        let dates = Self::sorted_dates_in_period(start_date, end_date);
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
        wtr.write_record(header).expect("error");
        for p in projects.iter() {
            for u in users.iter() {
                let durations: Vec<String> = dates
                    .iter()
                    .map(|d| {
                        let key = &RecordKey {
                            user: u.clone(),
                            project: p.clone(),
                            date: *d,
                        };
                        Self::format_duration_time(
                            summed_dur_time_by_project_user_date.get(key).unwrap_or(&0),
                        )
                    })
                    .collect();
                let row: Vec<String> = [
                    vec![
                        p.clone().unwrap_or(Self::NONE_PROJECT_LABEL.to_string()),
                        u.clone(),
                    ],
                    durations,
                ]
                .concat();
                wtr.write_record(row).expect("error");
            }
        }
        let data =
            String::from_utf8(wtr.into_inner().unwrap_or(vec![])).unwrap_or(String::from(""));
        data
    }
    /// Format duration time in msec to human-readable string
    ///
    /// e.g. 3601_000(msec) -> "1"
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

    fn sumup_durations(
        dur_time_by_project_user_date: &Vec<(RecordKey, u64)>,
    ) -> HashMap<RecordKey, u64> {
        // sum up duration time by RecordKey
        let summed_dur_time_by_project_user_date: HashMap<RecordKey, u64> =
            dur_time_by_project_user_date
                .into_iter()
                .cloned()
                .into_group_map()
                .into_iter()
                .map(|(k, v)| (k, v.iter().sum::<u64>()))
                .collect();
        summed_dur_time_by_project_user_date
    }
}
