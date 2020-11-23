extern crate csv;
extern crate clap;

use std::{io::prelude::*, fs::File, process::exit};
use csv::ReaderBuilder;
use clap::{Arg, App};

#[derive(Debug)]
struct Student {
    first_name: String,
    last_name: String,
    directory_id: String,
    uid: String,
    section: String,
    emails: Vec<String>,
}

fn main() {
    let args = App::new("UMD Roster Formatter")
        .version("1.0")
        .author("Vinnie Caprarola <vinnie@vcaprarola.me>")
        .about("Convert ELMS rosters to actual useful formats")
        .arg(Arg::with_name("roster")
             .short("r")
             .long("roster")
             .takes_value(true)
             .value_name("FILE")
             .help("A tab-separated roster downloaded from UMEG, with Email and Dir ID included")
             .multiple(true)
             .required(true))
        .arg(Arg::with_name("output-prefix")
             .short("o")
             .long("output-prefix")
             .takes_value(true)
             .value_name("FILE")
             .default_value("roster")
             .help("Prefix to assign to each output roster file"))
        .get_matches();

    /* Read each roster file */
    let students: Vec<_> = args.values_of("roster").unwrap().map(|roster| {
        /* Create a reader to read the roster file */
        let mut rdr = match ReaderBuilder::new().has_headers(false).delimiter(b'\t').from_path(roster) {
            Ok(rdr) => rdr,
            Err(_) => {
                eprintln!("Could not read file: {}", roster);
                exit(1);
            }
        };

        /* Map each record to a student instance */
        rdr.records().map(|record| match record {
            Ok(record) => {
                if record.len() < 5 {
                    eprintln!("Invalid input file: {} (not enough columns)", roster);
                    exit(1);
                }

                let name: Vec<_> = record[2].split(',').collect();
                let (first_name, last_name) = if name.len() == 2 {
                    (name[1].trim().into(), name[0].trim().into())
                } else if name.len() == 1 {
                    (name[0].trim().into(), "".into())
                } else {
                    eprintln!("Invalid name formatting: {} (in file: {})", &record[2], roster);
                    exit(1);
                };

                Student {
                    first_name,
                    last_name,
                    directory_id: record[3].trim().into(),
                    uid: record[1].trim().into(),
                    section: record[0].trim().into(),
                    emails: vec![record[4].trim().into()],
                }
            }
            Err(_) => {
                eprintln!("Invalid input file: {}", roster);
                exit(1);
            }
        }).collect::<Vec<_>>()
    }).fold(Vec::new(), |mut acc, mut x| {
        acc.append(&mut x); acc
    });

    /* Get file prefix */
    let prefix = args.value_of("output-prefix").unwrap();

    /* Print roster for Gradescope */
    {
        let mut file = File::create(format!("{}-gradescope.csv", prefix)).unwrap();

        file.write(b"First Name,Last Name,Email,SID,Section\n").unwrap();
        for s in students.iter() {
            file.write(format!("{},{},{}@umd.edu,{},{}\n", s.first_name, s.last_name, s.directory_id, s.uid, s.section).as_bytes()).unwrap();
        }
    }

    /* Print roster information for mapping uids to/from dids */
    {
        let mut file = File::create(format!("{}-idmap.csv", prefix)).unwrap();

        file.write(b"UID,DID,Name\n").unwrap();
        for s in students.iter() {
            file.write(format!("{},{},\"{} {}\"\n", s.uid, s.directory_id, s.first_name, s.last_name).as_bytes()).unwrap();
        }
    }
}
