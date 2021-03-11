//! Load and lookup individual students' extensions for the project.

use crate::roster::*;
use std::fs::File;

/// The types of errors that can be produced within and returned from this module.
#[derive(Debug)]
pub enum Error {
    ExtensionsReadError,
    ExtensionsFormatError(usize),
}

/// Represents a single row from the extensions CSV.
#[derive(serde::Deserialize)]
pub struct Extension {
    #[serde(rename = "UID")]
    pub uid: String,
    #[serde(rename = "Hours")]
    pub hours: u32,
}

/// Contains a list of extensions, each of which maps the student's UID to the number of hours
/// their deadline was extended.
pub struct ExtensionSet {
    extensions: Vec<Extension>,
}

impl ExtensionSet {
    /// Get an empty extension set.
    pub fn empty() -> ExtensionSet {
        ExtensionSet {
            extensions: Vec::new(),
        }
    }

    /// Given the name of the CSV file containing the list of extensions, loads them into an
    /// instance of `ExtensionSet`.  The format of the file must be UID,Hours, and the header line
    /// must be included at the top of the file.
    ///
    /// # Arguments
    ///
    /// * `file` - The name of the roster file.
    ///
    /// # Errors
    ///
    /// If the extensions file cannot be read (for example, if it doesn't exist or the appropriate
    /// permissions are not set), will return `ExtensionsReadError`.  If there is an error during
    /// deserialization, will return `ExtensionsFormatError` with the line number of the first
    /// error.
    pub fn load(file: &str) -> Result<ExtensionSet, Error> {
        // Open the file (this will fail if the file doesn't exist or we can't read it).
        let file = File::open(file).or(Err(Error::ExtensionsReadError))?;

        // Create a CSV reader over this file.
        let mut rdr = csv::Reader::from_reader(file);

        // Map each row into an Extension instance.
        let extensions: Result<Vec<_>, _> = rdr.deserialize().enumerate().map(|(i, row)| row.or(Err(Error::ExtensionsFormatError(i + 1)))).collect();

        // Create an ExtensionSet instance with all of those entries.
        Ok(ExtensionSet {
            extensions: extensions?,
        })
    }

    /// Look for an extension for a particular student.  There should not be multiple entries for a
    /// single student, so if there are, the first one will be returned.
    ///
    /// # Arguments
    ///
    /// * `student` - The student for whom to search for an extension.
    pub fn find(&self, student: &Student) -> Option<&Extension> {
        self.extensions.iter().find(|e| e.uid == student.uid)
    }
}
