# rosters

Use this for formatting the UMEG course roster(s) into more useful and readable formats.

## Usage

```
USAGE:
    cargo run -- [OPTIONS] --roster <FILE>...

  or, if you just have the binary:
    rosters [OPTIONS] --roster <FILE>...

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -o, --output-prefix <FILE>    Prefix to assign to each output roster file [default: roster]
        -r, --roster <FILE>...        A tab-separated roster downloaded from UMEG, with Email and Dir ID included
```

## Downloading UMEG Roster

To get started, to go the [UMEG Roster Download page](https://umeg.umd.edu/rosters/downRost).  Select the correct semester and class section at the top-right (be sure to click "Change" or "Display", resp., after changing the selection).  Then under "Delimited Roster", select "Email" and "Dir ID", set the Delimiter to "[Tab]", and click "Download Delimited".  This will be the input file you pass with the `--roster` argument.

## Output Rosters

The current implementation outputs two rosters:
- **roster-gradescope.csv**: This can be used to upload to Gradescope to add all students to the course.
- **roster-idmap.csv**: This is used for various other tools in this repository to convert UIDs to Directory IDs, or vice-versa.

## Example

`./rosters --roster CMSC330-01all.dlm`
