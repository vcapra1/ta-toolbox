# project-grades

Use this for computing project grades because Gradescope can't do stuff like extensions, late penalties, etc.  The first step is to export submissions from Gradescope.  The only file from this you need is `submission_metadata.yml`, but you have to download all the submissions to get this file.

## Usage

```
USAGE:
    cargo run -- [OPTIONS] --canonical <NAME> --due-date <YYYY-MM-DD HH:MM +/-ZZZZ> --output-dir <DIR> --roster <FILE> --submissions <FILE>...

  or, if you just have the binary:
    rust-grader [OPTIONS] --canonical <NAME> --due-date <YYYY-MM-DD HH:MM +/-ZZZZ> --output-dir <DIR> --roster <FILE> --submissions <FILE>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --canonical <NAME>                          The name of the submitter of the canonical solution
    -d, --due-date <YYYY-MM-DD HH:MM +/-ZZZZ>    Specify the project's due date
    -e, --extensions <FILE>                         A CSV file of extensions, in the format UID,Hours
    -p, --late-penalty <PENALTY>
            The penalty to apply to late submissions, expressed as a percentage (omit % sign) [default: 10.0]

    -l, --late-period <HOURS>
            The additional time students had to submit projects for late credit [default: 24.0]

    -o, --output-dir <DIR>                          Directory to put the output files in.
    -r, --roster <FILE>                             Specify the student roster to use
    -i, --student <UID>...
            Specify a student for whom to calculuate the project grade. If omitted, all grades are computed

    -s, --submissions <FILE>...                     The submission_metadata.yml file downloaded from Gradescope
```

## Example

```bash
./rust-grader --due-date "2020-02-11 23:59 -0500" --roster roster.csv --submissions p1/submission_metadata.yml --submissions p1/extended.yml --extensions p1/extensions.csv --canonical "Vincent Caprarola" -o p1
```

This will read two `submission_metadata.yml` files (one for the extended version), a list of extensions (`extensions.csv`), and output into the `p1` directory.  When specifying the due date you have to specify the time zone (e.g. `-0500`), which should be the same as the one on Gradescope in the project settings.

The canonical should be a known working submission (should be a submission by an instructor/TA).  Test names must be unique (this is a limitation of the grades server).

Also, to just compute a single student's grades, add an option `--student` with their UID, for example: `--student 123456789`.

You might get a warning about irregular test states (e.g. testing harness started).  This means at the time of export, that test was still running and had not finished.  These submissions will be ignored.  If you want those submissions to be included, wait for them to finish testing and re-export the submissions.

## Rosters

The format for the roster CSV is below:

```csv
UID,DID
123456789,umdterp
987654321,testudo
```

You can use the roster tool in this repository to generate rosters of this format (use the `roster-idmap.csv` file).  Note that this generated CSV file will also include a "Name" column, but it will still work as that column is optional.

## Extensions

If there are extensions, you can supply them to the program with a CSV.  It should consist of two columns: student ID and number of hours for the extension.  For example:

```csv
UID,Hours
123456789,24
987654321,72
```

This gives the first student a 1-day extension and the second a 3-day extension.

## Notes

- The program considers assignments submitted up to 5 minutes after the deadline to be on-time.  This is because Gradescope does not mark assignments as "Late" if they are submitted within 5 minutes after the deadline.
