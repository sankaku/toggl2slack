use crate::toggl::RecordKey;
use chrono::prelude::*;
use csv::WriterBuilder;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub struct MessageCreator {}
impl MessageCreator {
    const NONE_PROJECT_LABEL: &'static str = "EmptyProject";

    /// Formats project-duration vector to string
    ///
    /// TODO: The order of users is not conserved. Use BTreeMap instead of HashMap.
    pub fn get_project_message(
        &self,
        project_times_by_user: &HashMap<String, Vec<(Option<String>, u64)>>,
        begin_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let title = self.get_project_message_title(&begin_date, &end_date);
        project_times_by_user
            .iter()
            .fold(String::from(title), |acc, (u, project_times)| {
                acc + &self.get_project_message_user_entry(&u, &project_times)
            })
    }

    fn get_project_message_title(&self, begin_date: &NaiveDate, end_date: &NaiveDate) -> String {
        format!(
            "*Toggl summary report* [{begin}-{end}]\n",
            begin = begin_date.format("%Y/%m/%d"),
            end = end_date.format("%Y/%m/%d"),
        )
    }

    fn get_project_message_user_entry(
        &self,
        user: &String,
        project_times: &Vec<(Option<String>, u64)>,
    ) -> String {
        format!(
            "\n*{name}*\n\n```{project_times_text}```",
            name = user,
            project_times_text = project_times
                .iter()
                .fold(String::from(""), |acc, (p, dur)| {
                    acc + &format!(
                        "{project}: {time}h\n",
                        project = p.clone().unwrap_or(Self::NONE_PROJECT_LABEL.to_string()),
                        time = self.format_duration_time(dur),
                    )
                })
        )
    }

    /// Returns all dates between `begin_date` and `end_date`
    ///
    /// The dates are sorted ascendingly.
    fn get_sorted_dates_in_period(
        &self,
        begin_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> Vec<NaiveDate> {
        if begin_date > end_date {
            panic!("Wrong input")
        };
        begin_date
            .iter_days()
            .take_while(|x| x <= &end_date)
            .collect()
    }

    /// Returns report text csv-formatted
    ///
    /// e.g. (blanks are inserted for visibility here)
    /// Project,   User, 2020/12/01, 2020/12/02, 2020/12/03, ...
    /// projectA, Alice,  0.5,     1,     0, ...
    /// projectA,   Bob,    0,     5,     2, ...
    pub fn create_text_for_csv(
        &self,
        dur_time_by_project_user_date: &Vec<(RecordKey, u64)>,
        begin_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let summed_dur_time_by_project_user_date =
            self.sumup_durations(&dur_time_by_project_user_date);
        let dates = self.get_sorted_dates_in_period(begin_date, end_date);
        let projects: HashSet<Option<String>> = summed_dur_time_by_project_user_date
            .iter()
            .map(|(k, _)| k.project.clone())
            .collect();
        let users: HashSet<String> = summed_dur_time_by_project_user_date
            .iter()
            .map(|(k, _)| k.user.clone())
            .collect();

        self.write_csv(
            &users,
            &projects,
            &dates,
            &summed_dur_time_by_project_user_date,
        )
    }

    fn write_csv(
        &self,
        users: &HashSet<String>,
        projects: &HashSet<Option<String>>,
        dates: &Vec<NaiveDate>,
        dur_times_for_record_key: &HashMap<RecordKey, u64>,
    ) -> String {
        let dates_str: Vec<String> = dates
            .iter()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .collect();

        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        let header: Vec<String> = [
            vec!["Project".to_string(), "User".to_string()],
            dates_str.clone(),
        ]
        .concat();
        wtr.write_record(header).expect("error");

        for p in projects.iter().sorted() {
            for u in users.iter().sorted() {
                let durations: Vec<String> = self
                    .search_duration_time(&dates, &u, &p, &dur_times_for_record_key)
                    .iter()
                    .map(|d| self.format_duration_time(d))
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
        String::from_utf8(wtr.into_inner().unwrap_or(vec![])).unwrap_or(String::from(""))
    }

    /// Searches the duration time for the given `user` and `project` in all the date of `dates`
    ///
    /// If is doesn't exists, 0 is returned for the date.
    fn search_duration_time(
        &self,
        dates: &Vec<NaiveDate>,
        user: &String,
        project: &Option<String>,
        dur_times_for_record_key: &HashMap<RecordKey, u64>,
    ) -> Vec<u64> {
        dates
            .iter()
            .map(|d| {
                let key = RecordKey {
                    user: user.clone(),
                    project: project.clone(),
                    date: *d,
                };
                *dur_times_for_record_key.get(&key).unwrap_or(&0)
            })
            .collect()
    }

    /// Formats duration time in msec to human-readable string
    ///
    /// e.g. 3600_000(msec) -> "1"
    fn format_duration_time(&self, msec: &u64) -> String {
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

    /// Sums up duration times per combination of `RecordKey`
    fn sumup_durations(
        &self,
        dur_time_by_project_user_date: &Vec<(RecordKey, u64)>,
    ) -> HashMap<RecordKey, u64> {
        dur_time_by_project_user_date
            .into_iter()
            .cloned()
            .into_group_map()
            .into_iter()
            .map(|(k, v)| (k, v.iter().sum()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_project_message_must_work_when_there_is_only_one_user() {
        let mc = MessageCreator {};

        let user = "Alice".to_string();
        let project1 = Some("ProjectA".to_string());
        let project2 = Some("ProjectB".to_string());
        let dur1 = 3600_000;
        let dur2 = 7200_000;
        let project_times_by_user: HashMap<String, Vec<(Option<String>, u64)>> = [(
            user.clone(),
            vec![(project1.clone(), dur1), (project2.clone(), dur2)],
        )]
        .iter()
        .cloned()
        .collect();
        let begin_date = NaiveDate::from_ymd(2020, 12, 1);
        let end_date = NaiveDate::from_ymd(2020, 12, 31);

        let actual = mc.get_project_message(&project_times_by_user, &begin_date, &end_date);
        let expected = format!(
            "{}{}",
            "*Toggl summary report* [2020/12/01-2020/12/31]\n",
            "\n*Alice*\n\n```ProjectA: 1h\nProjectB: 2h\n```",
        );
        assert_eq!(actual, expected)
    }

    #[test]
    fn get_project_message_must_return_text_ordered_by_user_and_project_when_there_are_two_users() {
        let mc = MessageCreator {};

        let user1 = "Alice".to_string();
        let user2 = "Bob".to_string();
        let project1 = Some("ProjectA".to_string());
        let project2 = Some("ProjectB".to_string());
        let dur1 = 3600_000;
        let dur2 = 7200_000;
        let project_times_by_user: HashMap<String, Vec<(Option<String>, u64)>> = [
            (
                user2.clone(),
                vec![(project1.clone(), dur2), (project2.clone(), dur1)],
            ),
            (
                user1.clone(),
                vec![(project1.clone(), dur1), (project2.clone(), dur2)],
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let begin_date = NaiveDate::from_ymd(2020, 12, 1);
        let end_date = NaiveDate::from_ymd(2020, 12, 31);

        let actual = mc.get_project_message(&project_times_by_user, &begin_date, &end_date);
        let expected = format!(
            "{}{}{}",
            "*Toggl summary report* [2020/12/01-2020/12/31]\n",
            "\n*Alice*\n\n```ProjectA: 1h\nProjectB: 2h\n```",
            "\n*Bob*\n\n```ProjectA: 2h\nProjectB: 1h\n```",
        );
        assert_eq!(actual, expected)
    }

    #[test]
    fn get_project_message_user_entry_must_return_text_for_the_given_user() {
        let mc = MessageCreator {};

        let user = "Alice".to_string();
        let project1 = Some("ProjectA".to_string());
        let project2 = Some("ProjectB".to_string());
        let dur1 = 3600_000;
        let dur2 = 7200_000;
        let project_times = vec![(project1.clone(), dur1), (project2.clone(), dur2)];

        let actual = mc.get_project_message_user_entry(&user, &project_times);
        let expected = "\n*Alice*\n\n```ProjectA: 1h\nProjectB: 2h\n```";

        assert_eq!(actual, expected)
    }

    #[test]
    fn get_project_message_title_must_return_title_text() {
        let mc = MessageCreator {};

        let begin_date = NaiveDate::from_ymd(2020, 12, 1);
        let end_date = NaiveDate::from_ymd(2021, 12, 1);

        let actual = mc.get_project_message_title(&begin_date, &end_date);
        let expected = "*Toggl summary report* [2020/12/01-2021/12/01]\n";

        assert_eq!(actual, expected)
    }

    #[test]
    fn get_sorted_dates_in_period_must_return_dates_in_the_given_period() {
        let mc = MessageCreator {};

        let begin = NaiveDate::from_ymd(2019, 12, 29);
        let end = NaiveDate::from_ymd(2020, 1, 2);

        let actual = mc.get_sorted_dates_in_period(&begin, &end);
        let expected = vec![
            NaiveDate::from_ymd(2019, 12, 29),
            NaiveDate::from_ymd(2019, 12, 30),
            NaiveDate::from_ymd(2019, 12, 31),
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 2),
        ];
        assert_eq!(actual, expected)
    }

    #[test]
    #[should_panic]
    fn get_sorted_dates_in_period_must_panic_if_end_is_prior_to_begin() {
        let mc = MessageCreator {};

        let begin = NaiveDate::from_ymd(2020, 12, 1);
        let end = NaiveDate::from_ymd(2020, 11, 30);

        mc.get_sorted_dates_in_period(&begin, &end);
    }

    #[test]
    fn write_csv_must_return_() {
        let mc = MessageCreator {};

        let user1 = "Alice".to_string();
        let user2 = "Bob".to_string();
        let users: HashSet<String> = [user1.clone(), user2.clone()].iter().cloned().collect();

        let project1 = Some("ProjectA".to_string());
        let project2 = Some("ProjectB".to_string());
        let projects: HashSet<Option<String>> = [project1.clone(), project2.clone()]
            .iter()
            .cloned()
            .collect();

        let date1 = NaiveDate::from_ymd(2020, 12, 1);
        let date2 = NaiveDate::from_ymd(2020, 12, 2);
        let dates = vec![date1, date2];
        let record_key1 = RecordKey {
            user: user1.clone(),
            project: project1.clone(),
            date: date1,
        };
        let record_key2 = RecordKey {
            user: user2.clone(),
            project: project2.clone(),
            date: date2,
        };
        let dur1 = 3600_000;
        let dur2 = 3600_000;
        let dur_times_for_record_key: HashMap<RecordKey, u64> =
            [(record_key1, dur1), (record_key2, dur2)]
                .iter()
                .cloned()
                .collect();

        let actual = mc.write_csv(&users, &projects, &dates, &dur_times_for_record_key);
        let expected = format!(
            "{}\n{}\n{}\n{}\n{}\n",
            "Project,User,2020-12-01,2020-12-02",
            "ProjectA,Alice,1,0",
            "ProjectA,Bob,0,0",
            "ProjectB,Alice,0,0",
            "ProjectB,Bob,0,1",
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn search_duration_time_must_return_the_corresponding_duration_time_when_it_exists() {
        let mc = MessageCreator {};

        let user = "Alice".to_string();
        let project = Some("Project".to_string());
        let date = NaiveDate::from_ymd(2020, 12, 1);
        let dates = vec![date];
        let record_key = RecordKey {
            user: user.clone(),
            project: project.clone(),
            date: date,
        };
        let dur = 100;
        let dur_times_for_record_key: HashMap<RecordKey, u64> =
            [(record_key, dur)].iter().cloned().collect();

        let actual = mc.search_duration_time(&dates, &user, &project, &dur_times_for_record_key);
        let expected = vec![dur];

        assert_eq!(actual, expected)
    }

    #[test]
    fn search_duration_time_must_return_0_when_it_does_not_exist() {
        let mc = MessageCreator {};

        let user = "Alice".to_string();
        let project = Some("Project".to_string());
        let date = NaiveDate::from_ymd(2020, 12, 1);
        let dates = vec![date];
        let dur_times_for_record_key: HashMap<RecordKey, u64> = HashMap::new();

        let actual = mc.search_duration_time(&dates, &user, &project, &dur_times_for_record_key);
        let expected = vec![0];

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_1000_000_to_0() {
        let mc = MessageCreator {};

        let input = 1000_000 as u64;
        let actual = mc.format_duration_time(&input);
        let expected = "0".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_1800_000_to_05() {
        let mc = MessageCreator {};

        let input = 1800_000 as u64;
        let actual = mc.format_duration_time(&input);
        let expected = "0.5".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_3600_000_to_1() {
        let mc = MessageCreator {};

        let input = 3600_000 as u64;
        let actual = mc.format_duration_time(&input);
        let expected = "1".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn sumup_durations_must_sum_up_2_elements_which_have_the_same_record_key() {
        let mc = MessageCreator {};

        let record_key = RecordKey {
            user: "Alice".to_string(),
            project: Some("Project".to_string()),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let dur1 = 100;
        let dur2 = 200;

        let input = vec![(record_key.clone(), dur1), (record_key.clone(), dur2)];
        let actual = mc.sumup_durations(&input);
        let expected = [(record_key.clone(), dur1 + dur2)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        assert_eq!(actual, expected)
    }

    #[test]
    fn sumup_durations_must_not_sum_up_2_elements_which_have_different_record_keys() {
        let mc = MessageCreator {};

        let record_key1 = RecordKey {
            user: "Alice".to_string(),
            project: Some("Project".to_string()),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let record_key2 = RecordKey {
            user: "Alice".to_string(),
            project: Some("ProjectB".to_string()),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let dur1 = 100;
        let dur2 = 200;

        let input = vec![(record_key1.clone(), dur1), (record_key2.clone(), dur2)];
        let actual = mc.sumup_durations(&input);
        let expected = [(record_key1.clone(), dur1), (record_key2.clone(), dur2)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        assert_eq!(actual, expected)
    }
}
