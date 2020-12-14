# Gradescope Makeup Exams

As of now, there is no practical way to offer a makeup exam on Gradescope.  This may (and hopefully will) change in the future, but in the meantime I have been using this workaround.

- Navigate to the "Assignments" page using the left navigation bar, and then at the bottom click "Duplicate Assignment".  Select the assignment you wish to duplicate and set an appropriate name, such as "Exam 1 (Makeup)".
- Once on the page for the new assignment, click "Settings" on the left navigation bar, set the release and due dates appropriately, and set the time limit to 0.  The form will not let you submit with a value of 0 there, so open the developer tools (Ctrl+Shift+I), find the input field for the maximum time permitted, and either change the min attribute to 0, or remove it altogether.  Now you will be able to submit the form.
- Navigate to the "Extensions" page for the assignment.  Click "Add an extension" and select "Add ____ minutes".  Here, set the amount of time you want the student to have.  For example, if the exam is 120 minutes, you would enter 120 minutes here (unless the student gets other accomodations, then you will have to do those calculations manually).

Note that the assignment will appear in all students' lists, but any other student who attempts to start the assignment will be immediately kicked out as their 0-minute time limit expires immediately.
