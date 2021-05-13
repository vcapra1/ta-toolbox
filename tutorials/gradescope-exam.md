# Creating an Online Exam on Gradescope

This tutorial walks through the best process for setting up an exam on Gradescope.

## Create a sandbox course

At the time of writing, Gradescope doesn't provide a way to hide an assignment from students while also letting TAs do "trial runs".  The workaround is to create an entirely new course on Gradescope, which I will refer to as the "sandbox" course.  In this course you can add TAs as students, so that they can see what the exam will look like to students and do trial runs.

Later, when the exam is finalized, you can copy it over to the main course using the "Duplicate Assignment" function (which allows copying assignments across courses).

## Create the assignment

In the sandbox course, navigate to "Assignments" and click "Create Assignment".  Select "Online Assignment", and click "Next".  Give the assignment a name, release date, due date (those don't matter so much right now, but make sure it's available for the duration of testing), set a time limit, and click "Create Assignment".

Now you can create the questions.  At the time of writing, you can add new sections between existing sections, but you **cannot** add a new section at the beginning or reorder sections.  I would leave a section or two blank at the beginning in case you want to add an Introduction section later.  Otherwise, you'd have to create a new section after section 1 manually copy the entire section 1 into the new section 2.  It's not fun.

## Copy to the main course

**Before** copying the exam to the main course, set the release date, due date, and time limit as appropriate.  These will be copied over as-is to the main course, so if you leave it open as it was during testing, it will be open when you copy it over.

Once you are done writing the exam and testing it, go to the main course, navigate to "Assignments", and click "Duplicate Assignment".  In the popup, select the sandbox course and the exam you set up there, enter a name for the assignment, and click "Duplicate".

## Grading

The Gradescope Autograder will automatically grade single choice (radio buttons), multiple choice (checkboxes), and short answer questions.  For single choice problems, it is great.  For everything else, it is just an inconvenience.  it will only assign all-or-nothing scores for each question, so even if the short answer is *almost correct* or the student only selects one incorrect checkbox, it will give 0 points.  Also, if you have multiple input fields in a question (e.g. two sets of radio buttons), it will only give points if they are *both* correct, and 0 if either is incorrect.

To solve this, right before grading a question, delete all of the rubric items for that question to clear the grading progress.  Annoyingly, even after doing this, if a student submits, the autograder will re-create the default rubric items and try to grade them, so I recommend waiting until all submissions are in before grading.
