use std::fs::File;
use crate::roster::*;

pub fn load(roster: &mut Roster, file: &str) -> Result<(), Box<std::error::Error>> {
    let mut rdr = csv::Reader::from_reader(File::open(file)?);
    for record in rdr.records() {
        let record = record?;

        /* Find the student with the given uid */
        let uid = record.get(0).unwrap();
        let student = roster.0.iter_mut().find(|student| student.university_id == uid);

        if let Some(mut student) = student {
            student.extension = record.get(1).unwrap().parse::<f32>().unwrap();
        }
    }

    Ok(())
}
