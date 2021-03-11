//! Code for reading the students' information from the roster file and processing it for easy
//! lookup and storage of basic information.

use std::fs::File;

/// The types of errors that can be produced within and returned from this module.
#[derive(Debug)]
pub enum Error {
    RosterReadError,
    RosterFormatError(usize),
}

/// Represents a single student, with the information from the roster CSV file.
#[derive(serde::Deserialize, Hash, Debug)]
pub struct Student {
    // The student's name, which is optional.
    #[serde(rename = "Name")]
    pub name: Option<String>,
    // The student's University ID.
    #[serde(rename = "UID")]
    pub uid: String,
    // The student's Directory ID (their login username).
    #[serde(rename = "DID")]
    pub directory_id: String,
}

/// Represents the course roster which contains all of the students.  Basically imported
/// automatically from the CSV file.
pub struct Roster {
    pub students: Vec<Student>,
}

impl Roster {
    /// Given the name of the CSV file containing the roster, loads all of the students into an
    /// instance of `Roster`.  Note that the format of the roster is a CSV with the columns
    /// Name,UID,DID (or just UID,DID as the Name column is optional, and not used in this
    /// implementation).  The corresponding header must be included at the top of the file.
    ///
    /// # Arguments
    ///
    /// * `file` - The name of the roster file.
    ///
    /// # Errors
    ///
    /// If the roster file cannot be read (for example, if it doesn't exist or the appropriate
    /// permissions are not set), will return `RosterReadError`.  If there is an error during
    /// deserialization, will reuturn `RosterFormatError` with the line number of the first error.
    pub fn load(file: &str) -> Result<Roster, Error> {
        // Open the file (this will fail if the file doesn't exist or we can't read it).
        let file = File::open(file).or(Err(Error::RosterReadError))?;

        // Create a CSV reader over this file.
        let mut rdr = csv::Reader::from_reader(file);

        // Map each row into an instance of Student.
        let students: Result<Vec<_>, _> = rdr.deserialize().enumerate().map(|(i, row)| row.or(Err(Error::RosterFormatError(i + 1)))).collect();

        // Create a Roster with all of those entries.
        Ok(Roster {
            students: students?,
        })
    }

    /// Lookup a Student by their university id (UID).
    ///
    /// # Arguments
    ///
    /// * `uid` - The UID of the student to look for.
    pub fn find_student_by_uid(&self, uid: String) -> Option<&Student> {
        self.students.iter().find(|s| s.uid == uid)
    }
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Student {}
