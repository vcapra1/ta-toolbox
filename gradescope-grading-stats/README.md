# gradescope-grading-stats

Track TA's grading progress on Gradescope online assignments and exams.

## Usage

```
USAGE:
    cargo run -- [FLAGS] [OPTIONS] --assignment-id <ID> --course-id <ID> --email <EMAIL>

  or, if you just have the binary:
    gradescope-grading-stats [FLAGS] [OPTIONS] --assignment-id <ID> --course-id <ID> --email <EMAIL>

    FLAGS:
        -h, --help       Prints help information
        -s, --server     Run the program in server mode
        -V, --version    Prints version information

    OPTIONS:
        -a, --assignment-id <ID>     Gradescopes assignment ID
        -c, --course-id <ID>         Gradescope course ID
        -u, --email <EMAIL>          Specify the email you use to login to Gradescope
        -i, --interval <MINUTES>     The refresh interval, in minutes
        -P, --password <PASSWORD>    Specify the password you use to login to Gradescope (it is recommended that you do not
                                     supply this as an argument, but the option is here for scripting purposes)
        -p, --port <port>            The port to run the server on
```

## Example

First, go to the assignment page on Gradescope (the assignment **MUST** be an exam or online assignment).  The URL will look like this:

**https://www.gradescope.com/courses/XXXXXXXX/assignments/YYYYYYYY/grade**

Here, `XXXXXXXX` is the course id and `YYYYYYYY` is the assignment id.  These will be used below.

`./gradescope-grading-stats --course-id XXXXXXXX --assignment-id YYYYYYYY --email "vinnie@vcaprarola.me" --server --port 80 --interval 5`

This will start the server on port 80, refreshing the stats every 5 minutes.

Alternatively, you can run without the `--server` and subsequent options to just pull the stats once and print them to the console:

`./gradescope-grading-stats --course-id XXXXXXXX --assignment-id YYYYYYYY --email "vinnie@vcaprarola.me"`

## Recommendations

- Note that it takes about 30 seconds to pull all of the new stats, so setting the interval to 0 does not mean continuous updates.  Also, smaller values will hit Gradescope's servers harder, which isn't very nice, so I recommend setting it to 5 minutes.
- You can supply the password as an argument, but this will cause your password to be stored in plaintext in your shell history!  To avoid this, omit the argument and just enter your password when prompted.
- Gradescope's servers are in the Pacific Northwest, so if possible try to run this on a server near there.  Each update requires a number of HTTP requests linear in the number of questions on the assignment, so the latencies add up.
