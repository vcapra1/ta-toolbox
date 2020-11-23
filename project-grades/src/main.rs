extern crate csv;
#[macro_use]
extern crate clap;
extern crate serde;
extern crate chrono;
extern crate time;

mod roster;
mod extensions;
mod submissions;

use roster::*;
use clap::App;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Match the command line arguments */
    let yaml = load_yaml!("args.yml");
    let args = App::from_yaml(yaml).get_matches();

    /* Get roster data */
    eprintln!("[*] Loading roster");
    let mut roster = Roster::load(args.value_of("roster").unwrap())?;

    /* Get extension data and update roster */
    if let Some(ref file) = args.value_of("extensions") {
        extensions::load(&mut roster, file)?;
    }

    /* Get submissions */
    eprintln!("[*] Loading submissions");
    let submissions = {
        /* List to add all submissions to */
        let mut submissions = Vec::new();

        /* Load each submission file */
        for in_file in args.values_of("submissions").unwrap() {
            let count = submissions::load(&mut submissions, in_file)?;
            eprintln!("[*] {} submissions loaded from {}", count, in_file);
        }

        submissions
    };

    /* Get the list of tests from the canonical submission */
    let tests = match submissions.iter().find(|sub| {
        sub.student_name == args.value_of("canonical").unwrap() && sub.was_activated
    }) {
        Some(t) => t.tests.clone(),
        None => {
            panic!("[!] No active submission found for the canonical account");
        }
    };

    /* Get list of students to include */
    let students_to_include = if let Some(students) = args.values_of("student") {
        Some(students.map(|x| {
            x.to_string()
        }).collect::<Vec<_>>())
    } else {
        None
    };

    /* Associate all submissions with a student */
    eprintln!("[*] Associating submissions with students");
    'outer: for submission in submissions {
//        println!("{}", submission.submission_id);
        /* Make sure the submissions test cases match the canonical's */
        for test in tests.iter() {
            if let Some(sub_test) = submission.tests.iter().find(|t| t.name == test.name) {
                if sub_test.max != test.max {
                    eprintln!("[!] Test {} point value incorrect for submission {}", test.name, submission.submission_id);
                    continue 'outer;
                }
            } else {
                /* Test not found */
                eprintln!("[!] Test {} not found for submission {} ({})", test.name, submission.submission_id, submission.time);
                continue 'outer;
            }
        }

        /* Get student ID for submission */
        let student_id = submission.student_id.clone();

        /* Check if specified */
        if let Some(ref students_to_include) = students_to_include {
            if !students_to_include.contains(&student_id) {
                continue;
            }
        }

        /* Add the submission */
        match roster.add_submission(submission) {
            Ok(_) => (),
            Err(_) => eprintln!("[!] Could not find student with id \"{}\"", student_id),
        }
    }

    if !args.is_present("gfa") {
        /* Get the due date */
        let due_date = DateTime::parse_from_str(args.value_of("due_date").unwrap(), "%Y-%m-%d %H:%M %z").unwrap().with_timezone(&Utc);

        /* Get best submission for each student */
        let mut file = File::create(format!("{}/grades.csv", args.value_of("output").unwrap()))?;
        for student in roster.0.iter() {
            if let Some((best, is_late)) = student.best_submission(0.1, due_date) {
                /* Yeet that to standard out */
                write!(file, "{}", best)?;

                /* If late, say so */
                if is_late {
                    write!(file, "{},*,*0.9,Late\n", student.directory_id)?;
                } else {
                    write!(file, "{},*,*1,\n", student.directory_id)?;
                }
            }
        }

        /* List tests for parts csv */
        let mut file = File::create(format!("{}/parts.csv", args.value_of("output").unwrap()))?;
    //    let canon_student = roster.0.iter().find(|s| s.name == args.value_of("canonical").unwrap()).unwrap();
    //    let canon_submission = canon_student.submissions.iter().find(|s| s.was_activated).unwrap();

    //    let mut tests = canon_submission.tests.clone();
        let mut tests = tests.clone();
        tests.sort_by(|a, b| {
            a.number.partial_cmp(&b.number).unwrap()
        });
        for t in tests.iter() {
            write!(file, "{},{}\n", t.name, t.max)?;
        }
    } else {
        /* Check if the student has any submissions over 20% */
        let mut file = File::create(format!("{}/gfa.csv", args.value_of("output").unwrap()))?;
        for student in roster.0.iter() {
            if student.passed_gfa() {
                write!(file, "{},pass\n", student.directory_id)?;
            } else {
                write!(file, "{},fail\n", student.directory_id)?;
            }
        }
    }

    Ok(())
}
