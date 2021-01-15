use crate::toggl::RecordKey;
use crate::values::{Duration, Project, ProjectRecords, User};
use chrono::prelude::*;
use csv::WriterBuilder;
use itertools::Itertools;
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct MessageCreator {}
impl MessageCreator {
    /// Formats project-duration vector to string
    pub fn get_project_message(
        &self,
        project_times_by_user: &ProjectRecords,
        begin_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let title = self.get_project_message_title(&begin_date, &end_date);
        project_times_by_user
            .value
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
        user: &User,
        project_times: &Vec<(Project, Duration)>,
    ) -> String {
        format!(
            "\n*{name}*\n\n```{project_times_text}```",
            name = user.to_string(),
            project_times_text = project_times.iter().fold(String::new(), |acc, (p, dur)| {
                acc + &format!(
                    "{project}: {time}h\n",
                    project = p.to_string(),
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
        dur_time_by_project_user_date: &Vec<(RecordKey, Duration)>,
        begin_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> String {
        let summed_dur_time_by_project_user_date =
            self.sumup_durations(&dur_time_by_project_user_date);
        let dates = self.get_sorted_dates_in_period(begin_date, end_date);
        let projects: BTreeSet<Project> = summed_dur_time_by_project_user_date
            .iter()
            .map(|(k, _)| k.project.clone())
            .collect();
        let users: BTreeSet<User> = summed_dur_time_by_project_user_date
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
        users: &BTreeSet<User>,
        projects: &BTreeSet<Project>,
        dates: &Vec<NaiveDate>,
        dur_times_for_record_key: &HashMap<RecordKey, Duration>,
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
                let row: Vec<String> = [vec![p.to_string(), u.to_string()], durations].concat();
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
        user: &User,
        project: &Project,
        dur_times_for_record_key: &HashMap<RecordKey, Duration>,
    ) -> Vec<Duration> {
        dates
            .iter()
            .map(|d| {
                let key = RecordKey {
                    user: user.clone(),
                    project: project.clone(),
                    date: *d,
                };
                *dur_times_for_record_key
                    .get(&key)
                    .unwrap_or(&Duration::new(0))
            })
            .collect()
    }

    /// Formats duration time in msec to human-readable string
    ///
    /// e.g. 3600_000(msec) -> "1"
    fn format_duration_time(&self, dur: &Duration) -> String {
        let msec = dur.value;
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
        dur_time_by_project_user_date: &Vec<(RecordKey, Duration)>,
    ) -> HashMap<RecordKey, Duration> {
        dur_time_by_project_user_date
            .into_iter()
            .cloned()
            .into_group_map()
            .into_iter()
            .map(|(k, v)| (k, Duration::new(v.iter().map(|dur| dur.value).sum())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_project_message_must_work_when_there_is_only_one_user() {
        let mc = MessageCreator {};

        let user = User::new("Alice");
        let project1 = Project::new(Some("ProjectA"));
        let project2 = Project::new(Some("ProjectB"));
        let dur1 = Duration::new(3600_000);
        let dur2 = Duration::new(7200_000);
        let project_times_by_user = ProjectRecords::new(
            [(
                user.clone(),
                vec![(project1.clone(), dur1), (project2.clone(), dur2)],
            )]
            .iter()
            .cloned()
            .collect(),
        );
        let begin_date = NaiveDate::from_ymd(2020, 12, 1);
        let end_date = NaiveDate::from_ymd(2020, 12, 31);

        let actual = mc.get_project_message(&project_times_by_user, &begin_date, &end_date);
        println!("{:?}", actual);
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

        let user1 = User::new("Alice");
        let user2 = User::new("Bob");
        let project1 = Project::new(Some("ProjectA"));
        let project2 = Project::new(Some("ProjectB"));
        let dur1 = Duration::new(3600_000);
        let dur2 = Duration::new(7200_000);
        let project_times_by_user = ProjectRecords::new(
            [
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
            .collect(),
        );
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

        let user = User::new("Alice");
        let project1 = Project::new(Some("ProjectA"));
        let project2 = Project::new(Some("ProjectB"));
        let dur1 = Duration::new(3600_000);
        let dur2 = Duration::new(7200_000);
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

        let user1 = User::new("Alice");
        let user2 = User::new("Bob");
        let users: BTreeSet<User> = [user1.clone(), user2.clone()].iter().cloned().collect();

        let project1 = Project::new(Some("ProjectA"));
        let project2 = Project::new(Some("ProjectB"));
        let projects: BTreeSet<Project> = [project1.clone(), project2.clone()]
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
        let dur1 = Duration::new(3600_000);
        let dur2 = Duration::new(3600_000);
        let dur_times_for_record_key: HashMap<RecordKey, Duration> =
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

        let user = User::new("Alice");
        let project = Project::new(Some("Project"));
        let date = NaiveDate::from_ymd(2020, 12, 1);
        let dates = vec![date];
        let record_key = RecordKey {
            user: user.clone(),
            project: project.clone(),
            date: date,
        };
        let dur = Duration::new(100);
        let dur_times_for_record_key: HashMap<RecordKey, Duration> =
            [(record_key, dur)].iter().cloned().collect();

        let actual = mc.search_duration_time(&dates, &user, &project, &dur_times_for_record_key);
        let expected = vec![dur];

        assert_eq!(actual, expected)
    }

    #[test]
    fn search_duration_time_must_return_0_when_it_does_not_exist() {
        let mc = MessageCreator {};

        let user = User::new("Alice");
        let project = Project::new(Some("Project"));
        let date = NaiveDate::from_ymd(2020, 12, 1);
        let dates = vec![date];
        let dur_times_for_record_key: HashMap<RecordKey, Duration> = HashMap::new();

        let actual = mc.search_duration_time(&dates, &user, &project, &dur_times_for_record_key);
        let expected = vec![Duration::new(0)];

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_1000_000_to_0() {
        let mc = MessageCreator {};

        let input = Duration::new(1000_000);
        let actual = mc.format_duration_time(&input);
        let expected = "0".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_1800_000_to_05() {
        let mc = MessageCreator {};

        let input = Duration::new(1800_000);
        let actual = mc.format_duration_time(&input);
        let expected = "0.5".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn format_duration_time_must_3600_000_to_1() {
        let mc = MessageCreator {};

        let input = Duration::new(3600_000);
        let actual = mc.format_duration_time(&input);
        let expected = "1".to_string();

        assert_eq!(actual, expected)
    }

    #[test]
    fn sumup_durations_must_sum_up_2_elements_which_have_the_same_record_key() {
        let mc = MessageCreator {};

        let record_key = RecordKey {
            user: User::new("Alice"),
            project: Project::new(Some("Project")),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let dur1 = Duration::new(100);
        let dur2 = Duration::new(200);

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
            user: User::new("Alice"),
            project: Project::new(Some("Project")),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let record_key2 = RecordKey {
            user: User::new("Alice"),
            project: Project::new(Some("ProjectB")),
            date: NaiveDate::from_ymd(2020, 12, 1),
        };
        let dur1 = Duration::new(100);
        let dur2 = Duration::new(200);

        let input = vec![(record_key1.clone(), dur1), (record_key2.clone(), dur2)];
        let actual = mc.sumup_durations(&input);
        let expected = [(record_key1.clone(), dur1), (record_key2.clone(), dur2)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        assert_eq!(actual, expected)
    }
}
