//! Code for importing submission data from the yaml file produced by Gradescope.

use crate::{roster::*, extensions::*};
use std::fs::File;
use serde_yaml::Value;
use chrono::{DateTime, NaiveDateTime, Utc, Duration, TimeZone};

/// The types of errors that can be produced within and returned from this module.
#[derive(Debug)]
pub enum Error {
    SubmissionReadError,
    SubmissionFormatError(Option<u64>, usize),
    InvalidDeadlineError,
}

// A single test case and result
#[derive(Clone, Debug)]
pub struct TestCase {
    pub name: String,
    pub number: f64,
    pub score: f64,
    pub max: f64,
}

/// Represents a single submission of the project.
#[derive(Debug)]
pub struct Submission<'r> {
    // The submission ID
    pub id: u64,
    // The student
    pub student: &'r Student,
    // Submission time
    pub time: DateTime<Utc>,
    // Individual test results
    pub tests: Vec<TestCase>,
    // Whether this was the active submission
    pub active: bool,
}

/// A collection of submissions for a particular assigment.
pub struct SubmissionSet<'r> {
    roster: &'r Roster,
    pub submissions: Vec<Submission<'r>>,
}

impl <'r> Submission<'r> {
    /// Convert the YAML results for a submission into a `Submission` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The submission ID
    /// * `student` - The student whose submission this is
    /// * `active` - Whether this was the active submission
    /// * `submission_yaml` - YAML value containing the submission data
    ///
    /// # Errors
    ///
    /// If there is an error during deserialization, will return `SubmissionFormatError` with the
    /// ID of the first invalid submission.
    fn load(id: u64, student: &'r Student, active: bool, submission_yaml: &Value) -> Result<Submission<'r>, Error> {
        // Get time of submission
        let time = if let Some(Value::String(time)) = submission_yaml.get(":created_at") {
            DateTime::<Utc>::from_utc(NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S.%f Z").or(Err(Error::SubmissionFormatError(Some(id), 0)))?, Utc)
        } else {
            Err(Error::SubmissionFormatError(Some(id), 1))?
        };

        if let Some(results) = submission_yaml.get(":results") {
            if let Some(Value::Sequence(tests)) = results.get("tests") {
                // Parse tests
                let tests: Result<Vec<_>, _> = tests.into_iter().map(|t| {
                    if let (Some(Value::String(name)), Some(Value::String(number)), Some(Value::Number(score)), Some(Value::Number(max))) = (t.get("name"), t.get("number"), t.get("score"), t.get("max_score")) {
                        if let Ok(number) = number.parse::<f64>() {
                            if score.is_f64() && max.is_f64() {
                                Ok(TestCase {
                                    name: name.to_string(),
                                    number,
                                    score: score.as_f64().unwrap(),
                                    max: max.as_f64().unwrap(),
                                })
                            } else {
                                Err(Error::SubmissionFormatError(Some(id), 2))
                            }
                        } else {
                            Err(Error::SubmissionFormatError(Some(id), 20))
                        }
                    } else {
                        Err(Error::SubmissionFormatError(Some(id), 3))
                    }
                }).collect();

                Ok(Submission {
                    id,
                    student,
                    time,
                    tests: tests?,
                    active,
                })
            } else {
                Err(Error::SubmissionFormatError(Some(id), 4))?
            }
        } else {
            Err(Error::SubmissionFormatError(Some(id), 5))?
        }
    }

    /// Compute the total score of this submission using the assignment's due date and any
    /// extensions given to this individual student.
    ///
    /// # Arguments
    ///
    /// * `deadlines` - A list of 2-tuples, each containing the deadline and the respective penalty
    /// for submitting before that deadline.  The first element of this list is the normal due
    /// date, and must have a penalty of 0.  There must be at least one element.  The penalty
    /// should be given as a float between 0 and 1, where 0 indicates no penalty, and 1 indicates
    /// no credit (maximum penalty).
    /// * `extension` - An extension, if applicable, to apply to this submission.  Passed as an
    /// Option.
    ///
    /// # Errors
    ///
    /// If the given set of deadlines is invalid (as described above), will return
    /// `InvalidDeadlineError`.
    pub fn score(&self, deadlines: &Vec<(DateTime<Utc>, f64)>, extension: Option<&Extension>) -> Result<f64, Error> {
        Ok(self.raw_score() * (1. - self.compute_penalty(deadlines, extension)?))
    }

    /// Compute the penalty for this submission
    ///
    /// # Arguments
    ///
    /// * `deadlines` - A list of 2-tuples, each containing the deadline and the respective penalty
    /// for submitting before that deadline.  The first element of this list is the normal due
    /// date, and must have a penalty of 0.  There must be at least one element.  The penalty
    /// should be given as a float between 0 and 1, where 0 indicates no penalty, and 1 indicates
    /// no credit (maximum penalty).
    /// * `extension` - An extension, if applicable, to apply to this submission.  Passed as an
    /// Option.
    ///
    /// # Errors
    ///
    /// If the given set of deadlines is invalid (as described above), will return
    /// `InvalidDeadlineError`.
    pub fn compute_penalty(&self, deadlines: &Vec<(DateTime<Utc>, f64)>, extension: Option<&Extension>) -> Result<f64, Error> {
        // Make sure the first deadline is valid
        if deadlines.len() == 0 || deadlines[0].1 != 0. {
            return Err(Error::InvalidDeadlineError);
        }

        // Figure out which period this submission falls under
        for (deadline, penalty) in deadlines.iter() {
            // Allow a 5-minute buffer, just like Gradescope does, and add given extension.
            let deadline = if let Some(ref extension) = extension {
                *deadline + Duration::seconds(300) + Duration::seconds((extension.hours * 3600) as i64)
            } else {
                *deadline + Duration::seconds(300)
            };
        
            if self.time <= deadline {
                return Ok(*penalty);
            }
        }

        // The submission did not fall before any deadlines, so the penalty is 100%.
        Ok(1.)
    }

    /// Compute the raw total score of this submission, not taking into account any deadlines or
    /// extensions.
    pub fn raw_score(&self) -> f64 {
        let (score, max) = self.tests.iter().fold((0., 0.), |(a_s, a_m), x| (a_s + x.score, a_m + x.max));
        score / max
    }

    /// Validate this submission against a canonical submission.  This ensures the tests names and
    /// max scores are identical between the two submissions.  Returns true if this is a valid
    /// submission, false if not.
    ///
    /// # Arguments
    ///
    /// * `canonical` - The canonical submission against which to validate
    pub fn validate_with_canonical(&self, canonical: &Submission) -> bool {
        let mut tests = Vec::new();

        for t in self.tests.iter() {
            tests.push((t.name.clone(), t.number.clone(), t.max.clone()));
        }

        for t in canonical.tests.iter() {
            let k = (t.name.clone(), t.number.clone(), t.max.clone());
            if tests.contains(&k) {
                tests.remove(tests.iter().position(|x| *x == k).unwrap());
            } else {
                return false;
            }
        }

        tests.len() == 0
    }
}

impl <'r> SubmissionSet<'r> {
    /// Get an empty submission set.
    ///
    /// # Arguments
    ///
    /// * `roster` - The roster of students which will be used to assign a student to each
    pub fn new(roster: &'r Roster) -> SubmissionSet<'r> {
        SubmissionSet {
            roster,
            submissions: Vec::new(),
        }
    }

    /// Given the path to the submission_metadata.yml file downloaded from Gradescope, loads all of
    /// the submissions into `Submission` structs and adds them all to this `SubmissionSet`.
    ///
    /// # Arguments
    ///
    /// * `file` - The path to the YAML file, which is named submission_metadata.yml in the export
    /// submission.
    ///
    /// # Errors
    ///
    /// If the YAML file cannot be read (for example, if it doesn't exist or the appropriate
    /// permissions are not set), will return `SubmissionReadError`.  If there is an error during
    /// deserialization, will return `SubmissionFormatError` with the ID of the first invalid
    /// submission (or None if the error was not related to a particular submission).
    pub fn load(&mut self, file: &str) -> Result<(), Error> {
        // Open the file (this will fail if the file doesn't exist or we can't read it).
        let file = File::open(file).or(Err(Error::SubmissionReadError))?;

        // Parse the YAML file into a Value structure
        let yaml: Value = serde_yaml::from_reader(file).or(Err(Error::SubmissionFormatError(None, 6)))?;

        // Check that the data is the correct type (i.e. a mapping)
        if let Value::Mapping(mapping) = yaml {
            for (name, data) in mapping.into_iter() {
                // Extract the active submission id
                let submission_id = if let Value::String(name) = name {
                    if &name[0..11] == "submission_" {
                        *&name[11..].parse::<u64>().or(Err(Error::SubmissionFormatError(None, 7)))?
                    } else {
                        Err(Error::SubmissionFormatError(None, 8))?
                    }
                } else {
                    Err(Error::SubmissionFormatError(None, 9))?
                };

                // Check status
                if let Some(Value::String(status)) = data.get(":status") {
                    if status != "processed" {
                        if status != "failed" {
                            eprintln!("Testing not finished on {}", submission_id);
                        }
                        continue;
                    }
                } else {
                    Err(Error::SubmissionFormatError(Some(submission_id), 21))?
                }

                // Get associated student
                let student = if let Some(Value::Sequence(submitters)) = data.get(":submitters") {
                    // There should only be one entry
                    if submitters.len() != 1 {
                        Err(Error::SubmissionFormatError(Some(submission_id), 10))?
                    }

                    // Get the first entry
                    if let Value::Mapping(ref mapping) = submitters[0] {
                        if let Some(Value::String(id)) = mapping.get(&":sid".into()) {
                            if let Some(student) = self.roster.find_student_by_uid(id.clone()) {
                                student
                            } else {
                                eprintln!("No student found with id {}", id);
                                continue
                            }
                        } else {
                            Err(Error::SubmissionFormatError(Some(submission_id), 12))?
                        }
                    } else {
                        Err(Error::SubmissionFormatError(Some(submission_id), 13))?
                    }
                } else {
                    Err(Error::SubmissionFormatError(Some(submission_id), 14))?
                };

                // Parse the rest of the submission and add to list
                self.submissions.push(Submission::load(submission_id, student, true, &data)?);

                // Process sub-entries
                if let Some(Value::Sequence(history)) = data.get(":history") {
                    for data in history.into_iter() {
                        // Get the submission id
                        if let Some(Value::Number(submission_id)) = data.get(":id") {
                            let submission_id = submission_id.as_i64().ok_or(Error::SubmissionFormatError(None, 15))? as u64;

                            // Make sure its done
                            if let Some(Value::String(status)) = data.get(":status") {
                                if status != "processed" {
                                    if status != "failed" {
                                        eprintln!("Testing not finished on {}", submission_id);
                                    }
                                    continue;
                                }
                            } else {
                                Err(Error::SubmissionFormatError(Some(submission_id), 21))?
                            }

                            // Parse the rest of the submission and add to list
                            self.submissions.push(Submission::load(submission_id, student, false, data)?);
                        } else {
                            Err(Error::SubmissionFormatError(Some(submission_id), 16))?
                        }
                    }
                } else {
                    Err(Error::SubmissionFormatError(Some(submission_id), 17))?
                }
            }

            Ok(())
        } else {
            return Err(Error::SubmissionFormatError(None, 18));
        }
    }

    /// Find the most recent submission for a particular student before the given timestamp, if
    /// provided.  If no timestamp is provided, the most recent submission will be returned.  Will
    /// return None if no applicable submissions are found.
    ///
    /// # Arguments
    ///
    /// * `student` - The student whose submissions we should look for
    /// * `before` - If provided, a timestamp for which only submissions prior to it will be
    /// considered
    pub fn get_latest_submission<Z: TimeZone>(&self, student: &Student, before: Option<&DateTime<Z>>) -> Option<&Submission> {
        let mut latest: Option<&Submission> = None;

        for submission in self.submissions.iter() {
            if submission.student == student && (before.is_none() || submission.time <= *before.unwrap()) {
                if let Some(l) = latest {
                    // Compare to existing result
                    if submission.time > l.time {
                        latest = Some(submission);
                    }
                } else {
                    latest = Some(submission);
                }
            }
        }

        latest
    }

    /// Get the active submission for a particular student.  Will return None if the student has no
    /// submissions.
    ///
    /// # Arguments
    ///
    /// * `student` - The student whose submissions we should look for
    pub fn get_active_submission(&self, student: &Student) -> Option<&Submission> {
        for submission in self.submissions.iter() {
            if submission.student == student && submission.active {
                return Some(submission);
            }
        }

        None
    }
}

impl ToString for Submission<'_> {
    fn to_string(&self) -> String {
        let mut string = String::new();

        let mut tests = self.tests.clone();
        tests.sort_by(|a, b| {
            a.number.partial_cmp(&b.number).unwrap()
        });

        for t in tests {
            string.push_str(&format!("{},{},{},\n", self.student.directory_id, t.name, t.score));
        }

        string
    }
}
