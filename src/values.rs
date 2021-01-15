use chrono::prelude::*;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Add;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct User {
    pub value: String,
}
impl User {
    pub fn new<S: Into<String>>(value: S) -> Self {
        User {
            value: value.into(),
        }
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

struct UserVisitor;
impl<'de> Visitor<'de> for UserVisitor {
    type Value = User;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String for User")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(User::new(v))
    }
}
impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<User, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UserVisitor)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Project {
    pub value: Option<ProjectValue>,
}

impl Project {
    const NONE_PROJECT_LABEL: &'static str = "EmptyProject";

    pub fn new<S: Into<String>>(value: Option<S>) -> Self {
        Project {
            value: value.map(|x| ProjectValue { value: x.into() }),
        }
    }

    pub fn to_string(&self) -> String {
        self.clone()
            .value
            .map(|v| v.to_string())
            .unwrap_or(Self::NONE_PROJECT_LABEL.to_string())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProjectValue {
    pub value: String,
}

impl ProjectValue {
    pub fn new<S: Into<String>>(value: S) -> Self {
        ProjectValue {
            value: value.into(),
        }
    }

    pub fn to_string(&self) -> String {
        self.to_string()
    }
}

struct ProjectVisitor;
impl<'de> Visitor<'de> for ProjectVisitor {
    type Value = Project;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String for Project")
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let result_project_value = d.deserialize_str(ProjectValueVisitor);
        result_project_value.map(|v| Project { value: Some(v) })
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Project::new(None::<String>))
    }
}
impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> Result<Project, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(ProjectVisitor)
    }
}

struct ProjectValueVisitor;
impl<'de> Visitor<'de> for ProjectValueVisitor {
    type Value = ProjectValue;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("String for ProjectValue")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ProjectValue::new(v))
    }
}
impl<'de> Deserialize<'de> for ProjectValue {
    fn deserialize<D>(deserializer: D) -> Result<ProjectValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ProjectValueVisitor)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Duration {
    pub value: u64,
}
impl Duration {
    pub fn new(value: u64) -> Self {
        Duration { value }
    }
}
impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            value: self.value + other.value,
        }
    }
}

struct DurationVisitor;
impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("u64 for Duration")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration::new(v))
    }
}
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u64(DurationVisitor)
    }
}

#[derive(Debug)]
pub struct Period {
    begin: NaiveDate,
    end: NaiveDate,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectRecords {
    pub value: BTreeMap<User, Vec<(Project, Duration)>>,
}
impl ProjectRecords {
    pub fn new(value: BTreeMap<User, Vec<(Project, Duration)>>) -> Self {
        ProjectRecords { value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TmpJsonForUser {
        user: User,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TmpJsonForProject {
        project: Project,
    }

    #[test]
    fn this_json_must_be_deserialized_as_user() {
        let json = r#"{"user": "Alice"}"#;
        let actual = serde_json::from_str::<TmpJsonForUser>(json).unwrap();
        let expected = TmpJsonForUser {
            user: User {
                value: "Alice".to_string(),
            },
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn this_json_must_be_deserialized_as_project() {
        let json = r#"{"project": "ProjectA"}"#;
        let actual = serde_json::from_str::<TmpJsonForProject>(json).unwrap();
        let expected = TmpJsonForProject {
            project: Project {
                value: Some(ProjectValue {
                    value: "ProjectA".to_string(),
                }),
            },
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn this_json_must_be_deserialized_as_empty_project() {
        let json = r#"{"project": null}"#;
        let actual = serde_json::from_str::<TmpJsonForProject>(json).unwrap();
        let expected = TmpJsonForProject {
            project: Project { value: None },
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn this_empty_json_must_be_deserialized_as_empty_project() {
        let json = r#"{}"#;
        let actual = serde_json::from_str::<TmpJsonForProject>(json).unwrap();
        let expected = TmpJsonForProject {
            project: Project { value: None },
        };
        assert_eq!(actual, expected)
    }
}
