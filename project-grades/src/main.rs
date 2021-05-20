extern crate serde;
extern crate csv;
extern crate clap;

mod roster;
mod extensions;
mod submissions;

use roster::*;
use extensions::*;
use submissions::*;
use std::{fs::File, io::Write, collections::HashMap};
use chrono::{DateTime, Utc, Duration};
use clap::*;

fn main() {
    // Load command-line args
    let yaml = load_yaml!("args.yml");
    let args = App::from_yaml(yaml).get_matches();

    // Load the roster
    let roster = Roster::load(args.value_of("roster").unwrap()).unwrap();

    // Load all of the submissions
    let submissions = {
        let mut submissions = SubmissionSet::new(&roster);

        for in_file in args.values_of("submissions").unwrap() {
            submissions.load(in_file).unwrap();
        }

        submissions
    };

    // Load the extensions
    let extensions = if let Some(extensions_file) = args.value_of("extensions") {
        ExtensionSet::load(extensions_file).unwrap()
    } else {
        ExtensionSet::empty()
    };

    // Load the deadlines
    let deadlines = {
        let mut deadlines = Vec::new();

        // The first entry is just the normal deadline
        let due_date = DateTime::parse_from_str(args.value_of("due_date").unwrap(), "%Y-%m-%d %H:%M %z").unwrap().with_timezone(&Utc);
        deadlines.push((due_date, 0.));

        // Add the remaining deadlines
        match args.values_of("deadline") {
            Some(deadline_args) => for deadline in deadline_args {
                let parts: Vec<_> = deadline.split(",").collect();
                if parts.len() != 2 {
                    panic!("Invalid format for arg \"deadline\": {}", deadline);
                }

                // Get the hours and penalty
                let hours = parts[0].parse::<u32>().unwrap();
                let penalty = parts[1].parse::<f64>().unwrap();

                // Compute the new deadline
                let deadline = due_date + Duration::seconds((hours * 3600) as i64);

                deadlines.push((deadline, penalty));
            }
            _ => ()
        }

        deadlines
    };

    // Get output dir
    let output_dir = args.value_of("output").unwrap();

    // For each student, get their latest submission in each penalty period, as well as their
    // activated submission (which may be included in the former collection as well)
    let submission_candidates = {
        let mut submission_candidates: HashMap<&Student, (&Submission, Vec<Option<&Submission>>)> = HashMap::new();

        for student in roster.students.iter() {
            // Get the student's active submission
            if let Some(active) = submissions.get_active_submission(student) {
                // Get the student's latest submission in each penalty period
                let ext = if let Some(ref extension) = extensions.find(student) {
                    Duration::seconds(300) + Duration::seconds((extension.hours * 3600) as i64)
                } else {
                    Duration::seconds(300)
                };
                let latest = deadlines.iter().map(|(d, _)| submissions.get_latest_submission(student, Some(&(*d + ext)))).collect();

                // Add to the collection
                submission_candidates.insert(student, (active, latest));
            }
        }

        submission_candidates
    };

    // Compare the canonical submission to all of these submissions, ensuring that the tests match
    let canonical = {
        // Find the canonical submission
        let canonical_id = args.value_of("canonical").unwrap();
        let canonical_submission = submissions.get_active_submission(roster.find_student_by_uid(canonical_id.to_owned()).expect("No submitter with given ID for canonical found")).expect("No canonical submission found");

        // Make sure the score is 100
        if canonical_submission.raw_score() != 1.0 {
            panic!("Canonical submission did not receive full points.");
        }

        // Validate all other submissions against the canonical
        let mut invalid_submissions = Vec::new();

        for (_, (a, ls)) in submission_candidates.iter() {
            if !a.validate_with_canonical(&canonical_submission) {
                invalid_submissions.push(a);
            }
            for l in ls.iter() {
                if let Some(l) = l {
                    if !l.validate_with_canonical(&canonical_submission) {
                        invalid_submissions.push(l);
                    }
                }
            }
        }

        if invalid_submissions.len() > 0 {
            // Write invalid submission IDs to file
            let filename = format!("{}/invalid_submission_ids", output_dir);
            let mut file = File::create(&filename).unwrap();
            for invalid in invalid_submissions.iter() {
                write!(file, "{}\n", invalid.id).unwrap();
            }
            panic!("Some submissions were invalid; IDs written to {}", filename);
        }

        canonical_submission
    };

    // Of the submissions collected above, find the best scoring one for each student.  In the case
    // of a tie, prefer the active one, or the earliest submitted.
    let best_submissions: HashMap<&Student, &Submission> = submission_candidates.into_iter().map(|(student, (a, ls))| {
        let extension = extensions.find(student);
        let (mut best, mut best_score) = (a, a.score(&deadlines, extension).unwrap());
        for l in ls.iter() {
            if let Some(l) = l {
                let new_score = l.score(&deadlines, extension).unwrap();
                if new_score > best_score {
                    best = l;
                    best_score = new_score;
                }
            }
        }
        (student, best)
    }).collect();

    // Generate the parts.csv
    {
        let mut file = File::create(format!("{}/parts.csv", output_dir)).unwrap();
        let mut tests = canonical.tests.clone();
        tests.sort_by(|a, b| {
            a.number.partial_cmp(&b.number).unwrap()
        });
        for t in tests.iter() {
            write!(file, "{},{}\n", t.name, t.max).unwrap();
        }
    }

    // Generate the grades.csv
    {
        let mut file = File::create(format!("{}/grades.csv", output_dir)).unwrap();

        for (_, submission) in best_submissions.iter() {
            let penalty = submission.compute_penalty(&deadlines, extensions.find(submission.student)).unwrap();

            if penalty < 1. {
                write!(file, "{}", submission.to_string()).unwrap();

                if penalty != 0. {
                    write!(file, "{},*,*{},Late\n", submission.student.directory_id, 1. - penalty).unwrap();
                } else {
                    write!(file, "{},*,*1,\n", submission.student.directory_id).unwrap();
                }
            }
        }
    }
}
