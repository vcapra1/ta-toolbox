//! Methods and data structures for parsing submission metadata and storing the results in an
//! organized way, which will be more usable when computing the students' final grades for the
//! project.

use chrono::{DateTime, Utc};
use serde_yaml::Value;
use std::fs::File;

/// Represents a single project submission, with all information necessary for final score
/// computation.  Includes the time of the submission in UTC and a list of the test results.
pub struct Submission {
    pub submission_id: u64,
    pub student_name: String,
    pub student_id: String,
    pub student_did: Option<String>,
    pub time: DateTime<Utc>,
    pub total_score: f64,
    pub tests: Vec<TestCase>,
    pub was_activated: bool,
}

/// Represents a run of a single test case for a particular submission.  Includes the name of the
/// test, its number (for ordering), the score received by the submission, and the maximum number
/// of points for this test case.
#[derive(Clone)]
pub struct TestCase {
    pub name: String,
    pub number: f64,
    pub score: f64,
    pub max: f64,
}

/// An error type for this module, which will be returned by the load method so errors can be
/// handled further up.
#[derive(Debug)]
pub enum Error {
    YamlFileError(String),
    YamlStructureError(String),
}

/// Load the submissions from a file, and store all of the submissions found in the dest Vec.  This
/// will add to dest, so the function can be called many times to accumulate submission information
/// from many files.
///
/// Returns the number of submissions loaded on success, or an appropriate Error message on
/// failure.  If there is a failure, all submissions that were already found will still be added to
/// the dest Vec.
pub fn load(dest: &mut Vec<Submission>, file: &str) -> Result<usize, Error> {
    /* Open and parse the YAML file */
    let yaml: Value = serde_yaml::from_reader(File::open(file)?)?;

    /* Keep track of number added */
    let mut count = 0;

    if let Value::Mapping(mapping) = yaml {
        /* Iterate over all of the active submissions */
        for submission_set in mapping.into_iter() {
            count += process_submissions(dest, submission_set)?;
        }
    } else {
        Err(Error::YamlStructureError(String::from("YAML structure invalid: top-level should be a Mapping.")))?
    }

    Ok(count)
}

fn process_submissions(dest: &mut Vec<Submission>, (name, data): (Value, Value)) -> Result<usize, Error> {
    /* Get the active submission id from the key */
    let submission_id = if let Value::String(name) = name {
        /* Check that the submission name follows the expected format */
        if &name[0..11] == "submission_" {
            /* Get the numerical part of the name as the id */
            match &name[11..].parse::<u64>() {
                Ok(id) => Ok(*id),
                Err(_) => Err(Error::YamlStructureError(format!("YAML strucutre invalid: invalid submission name {}.", name))),
            }
        } else {
            Err(Error::YamlStructureError(format!("YAML strucutre invalid: invalid submission name {}.", name)))
        }
    } else {
        Err(Error::YamlStructureError(String::from("YAML structure invalid: key for each submission group should be a String.")))
    }?;

    /* Get info about submitter */
    let (student_name, student_id) = if let Some(Value::Sequence(submitters)) = data.get(":submitters") {
        /* Get the first entry (there should only be one */
        if let Value::Mapping(ref mapping) = submitters[0] {
            match (mapping.get(&":name".into()), mapping.get(&":sid".into())) {
                (Some(Value::String(name)), Some(Value::String(id))) => Ok((name.clone(), id.clone())),
                _ => Err(Error::YamlStructureError(format!("Invalid \":submitters\" field for submission {}.", submission_id))),
            }
        } else {
            Err(Error::YamlStructureError(format!("Empty \":submitters\" field for submission {}.", submission_id)))
        }
    } else {
        Err(Error::YamlStructureError(format!("No \":submitters\" field for submission {}.", submission_id)))
    }?;

    /* Keep track of number added */
    let mut count = 0;

    /* Get active submission */
    match process_submission(data.clone(), (student_name.clone(), student_id.clone()), Some(submission_id)) {
        Ok(Some(mut active)) => {
            active.was_activated = true;
            dest.push(active);
            count += 1;
        },
        Err(err) => {
            eprintln!("error caught: {:?}", err);
        },
        _ => ()
    }

    /* Get the rest of the submissions */
    if let Some(Value::Sequence(subs)) = data.get(":history") {
        for sub in subs.iter() {
            match process_submission(sub.clone(), (student_name.clone(), student_id.clone()), None) {
                Ok(Some(submission)) => {
                    dest.push(submission);
                    count += 1;
                },
                Err(err) => {
                    eprintln!("{:?}", err);
                },
                _ => ()
            }
        }
    }

    Ok(count)
}

fn process_submission(data: Value, (student_name, student_id): (String, String), submission_id: Option<u64>) -> Result<Option<Submission>, Error> {
    /* Get the submission id */
    let submission_id = if let Some(id) = submission_id {
        Ok(id)
    } else if let Some(Value::Number(id)) = data.get(":id") {
        match id.as_u64() {
            Some(id) => Ok(id),
            None => Err(Error::YamlStructureError(format!("{} is not a valid submission id.", id))),
        }
    } else {
        Err(Error::YamlStructureError(format!("Submission without an ID.")))
    }?;

    /* Check the status of the submission */
    if let Some(Value::String(status)) = data.get(":status") {
        if status != "processed" {
            if status != "failed" {
                eprintln!("[!] Irregular status on submission {}: {}", submission_id, status);
            }
            return Ok(None);
        }
    } else {
        return Ok(None);
    }

    /* Check if the submission timed out */
    if let Some(results) = data.get(":results") {
        if let Some(Value::String(output)) = results.get("output") {
            if output.contains("timed out") {
                return Ok(None);
            }
        }
    }

    /* Get time of submission */
    let time = if let Some(Value::String(time)) = data.get(":created_at") {
        let mut time = time.clone();
        /* Check for "Z" in time zone */
        if &time[time.len() - 1..] == "Z" {
            /* Replace the Z with +0000 */
            time.pop();
            time.push_str("+0000");
            
            match DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S.%f %z") {
                Ok(dt) => Ok(dt.with_timezone(&Utc)),
                Err(_) => Err(Error::YamlStructureError(format!("Invalid time format for submission {}: {}", submission_id, time))),
            }
        } else {
            Err(Error::YamlStructureError(format!("Invalid time format for submission {}: {}", submission_id, time)))
        }
    } else {
        Err(Error::YamlStructureError(format!("No \":created_at\" field for submission {}.", submission_id)))
    }?;

    /* Get score */
    let score = if let Some(Value::Number(score)) = data.get(":score") {
        if score.is_f64() {
            Ok(score.as_f64().unwrap())
        } else if score.is_i64() {
            Ok(score.as_i64().unwrap() as f64)
        } else if score.is_u64() {
            Ok(score.as_u64().unwrap() as f64)
        } else {
            Err(Error::YamlStructureError(format!("Invalid score for submission {}", submission_id)))
        }
    } else {
        Err(Error::YamlStructureError(format!("Invalid or missing score for submission {}", submission_id)))
    }?;

    /* Get test results */
    let test_results = if let Some(cases) = data.get(":results") {
        if let Some(Value::Sequence(cases)) = cases.get("tests") {
            let mut err = None;

            let test_results = cases.iter().map(|case| {
                /* Get the name of the test case */
                if let Some(Value::String(name)) = case.get("name") {
                    let name = name.to_string();
                    /* Get the number of the test case */
                    if let Some(Value::String(number)) = case.get("number") {
                        let number = number.to_string();
                        if let Ok(number) = number.parse::<f64>() {
                            /* Get the score */
                            if let Some(Value::Number(score)) = case.get("score") {
                                if score.is_f64() {
                                    let score = score.as_f64().unwrap();
                                    /* Get the max score */
                                    if let Some(Value::Number(max)) = case.get("max_score") {
                                        if max.is_f64() {
                                            let max = max.as_f64().unwrap();

                                            /* Create the test case */
                                            Some(TestCase { name, number, score, max })
                                        } else {
                                            err.replace(Error::YamlStructureError(format!("Invalid max_score field for test case on submission {}", submission_id)));
                                            None
                                        }
                                    } else {
                                        err.replace(Error::YamlStructureError(format!("No max_score field for test case on submission {}", submission_id)));
                                        None
                                    }
                                } else {
                                    err.replace(Error::YamlStructureError(format!("Invalid score field for test case on submission {}", submission_id)));
                                    None
                                }
                            } else {
                                err.replace(Error::YamlStructureError(format!("No score field for test case on submission {}", submission_id)));
                                None
                            }
                        } else {
                            err.replace(Error::YamlStructureError(format!("Invalid number {} for test case on submission {}", number, submission_id)));
                            None
                        }
                    } else {
                        err.replace(Error::YamlStructureError(format!("No number field for test case on submission {}", submission_id)));
                        None
                    }
                } else {
                    err.replace(Error::YamlStructureError(format!("No name field for test case on submission {}", submission_id)));
                    None
                }
            }).collect::<Vec<_>>();

            if let Some(err) = err {
                Err(err)
            } else {
                Ok(test_results.into_iter().map(|c| c.unwrap()).collect::<Vec<_>>())
            }
        } else {
            Err(Error::YamlStructureError(format!("Invalid test results structure for submission {} (should be a Sequence).", submission_id)))
        }
    } else {
        Err(Error::YamlStructureError(format!("Missing test results for submission {}", submission_id)))
    }?;

    Ok(Some(Submission {
        submission_id,
        student_name,
        student_id,
        student_did: None,
        time,
        total_score: score,
        tests: test_results,
        was_activated: false,
    }))
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::YamlFileError(format!("{}", err))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::YamlStructureError(format!("{}", err))
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Error loading submissions"
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

impl std::fmt::Display for Submission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tests = self.tests.clone();
        tests.sort_by(|a, b| {
            a.number.partial_cmp(&b.number).unwrap()
        });

        for t in tests {
            write!(f, "{},{},{},\n", self.student_did.clone().unwrap(), t.name, t.score)?;
        }

        Ok(())
    }
}
