# Gradescope Exam Grading Stats

It is useful for everyone to be able to see the current grading stats for a project, which include how many of each question each TA has graded.  See (the grading stats directory)[../gradescope-grading-stats] for setup instructions. Usually I just run this program on a cheap VPS (I use Vultr, usually only cost like $2 to have it running for the entire duration of grading).  Just make sure it has at least 2GB of RAM, I ran out of memory when compiling on less than that.  You also have to install Rust, `libssl-dev` and `pkgconfig` so the program can compile.

If you append `&!` to the command, the process will be disowned so it will keep running if you close the SSH session.  To stop it, you'll have to use `kill -9 <pid>`, use `ps aux | grep gradescope` to find the PID.
