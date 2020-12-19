use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Add;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct User {
    pub value: String,
}
impl User {
    pub fn new<S: Into<String>>(value: S) -> Self {
        User {value: value.into()}
    }
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Project {
    pub value: Option<String>,
}
impl Project {
    const NONE_PROJECT_LABEL: &'static str = "EmptyProject";

    // pub fn new<S: Into<String>>(value: Option<S>) -> Self {
    pub fn new(value: Option<&str>) -> Self {
        Project {value: value.map(|x| x.into())}
    }

    pub fn to_string(&self) -> String {
        match &self.value {
            Some(v) => v.to_string(),
            None => Self::NONE_PROJECT_LABEL.to_string(),
        }
        // self.value.map(|x| x.to_string()).unwrap_or(Self::NONE_PROJECT_LABEL.to_string())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Duration {
    pub value: u64,
}
impl Duration {
    pub fn new(value: u64) -> Self {
        Duration {value}
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
        ProjectRecords {value}
    }
}