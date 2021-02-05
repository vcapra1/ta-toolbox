use std::fs::File;
use serde::Deserialize;
use crate::submissions::{self, *};
use chrono::{DateTime, Utc, Duration};

pub struct Roster(pub Vec<Student>);

#[derive(Deserialize)]
pub struct Student {
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "UID")]
    pub university_id: String,
    #[serde(rename = "DID")]
    pub directory_id: String,
    #[serde(skip)]
    pub extension: f32,
    #[serde(skip)]
    pub submissions: Vec<submissions::Submission>,
}

#[derive(Debug)]
pub enum Error {
    UserNotFound(String),
}

impl Roster {
    pub fn load(file: &str) -> Result<Roster, Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_reader(File::open(file)?);
        Ok(Roster(rdr.deserialize().map(|record| record.unwrap()).collect()))
    }

    pub fn add_submission(&mut self, mut submission: submissions::Submission) -> Result<(), Error> {
        let id = submission.student_id.clone();

        if let Some(student) = self.0.iter_mut().find(|st| st.university_id == id) {
            submission.student_did = Some(student.directory_id.clone());
            student.submissions.push(submission);
            Ok(())
        } else {
            Err(Error::UserNotFound(id))
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Error associating submission with student"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

impl Student {
    pub fn passed_gfa(&self) -> bool {
        /* Find any submission over 20% */
        for sub in self.submissions.iter() {
            if sub.total_score > 20f64 {
                return true;
            }
        }

        return false;
    }

    pub fn best_submission(&self, late_penalty: f64, late_period: f64, due_date: DateTime<Utc>) -> Option<(&Submission, bool)> {
        /* Get this student's personal due date (by applying the extension) */
        let due_date = due_date + Duration::minutes((self.extension * 60.0) as i64 + 5);

        /* Get this student's late due date */
        let late_due_date = due_date + Duration::minutes((late_period * 60.0) as i64);

        /* Partition the submissions into groups based on the time they were submitted */
        let mut on_time_subs = Vec::new();
        let mut late_subs = Vec::new();
        let mut active_subs = Vec::new();

        for sub in self.submissions.iter() {
            if sub.time <= due_date {
                on_time_subs.push(sub);
            } else if sub.time <= late_due_date {
                late_subs.push(sub);
            }

            if sub.was_activated {
                active_subs.push(sub);
            }
        }

        /* Find the latest on-time submission */
        on_time_subs.sort_by(|a, b| {
            a.time.partial_cmp(&b.time).unwrap()
        });
        late_subs.sort_by(|a, b| {
            a.time.partial_cmp(&b.time).unwrap()
        });

        /* Get all final candidates */
        let mut candidates = Vec::new();
        if let Some(s) = on_time_subs.pop() {
            candidates.push(s);
        }
        if let Some(s) = late_subs.pop() {
            candidates.push(s);
        }
        candidates.append(&mut active_subs);

        /* Get the final candidate with the highest score */
        candidates.sort_by(|a, b| {
            /* Get the score for the submission */
            let a_score = {
                if a.time <= due_date {
                    a.total_score
                } else if a.time <= late_due_date {
                    a.total_score * (1.0 - late_penalty)
                } else {
                    0.0
                }
            };

            let b_score = {
                if b.time <= due_date {
                    b.total_score
                } else if b.time <= late_due_date {
                    b.total_score * (1.0 - late_penalty)
                } else {
                    0.0
                }
            };

            a_score.partial_cmp(&b_score).unwrap()
        });

        if let Some(best) = candidates.pop() {
            if best.time <= due_date {
                Some((best, false))
            } else if best.time <= late_due_date {
                Some((best, true))
            } else {
                None
            }
        } else {
            None
        }
    }
}
