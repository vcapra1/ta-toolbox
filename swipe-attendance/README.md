# swipe-attendance

Take attendance more easily during exams by swiping the ID cards.

## Usage

```
Usage: takeAttendance.rb <courseID> <assignmentID> <cookies> <logFile>
```

## How to Use

It's recommended that you have an ID card reader like [this](https://smile.amazon.com/2xhome-Magnetic-Registry-Register-Quickbook/dp/B00E85TH9I/ref=asc_df_B00E85TH9I/?tag=hyprod-20&linkCode=df0&hvadid=167140485757&hvpos=&hvnetw=g&hvrand=17502797304766973073&hvpone=&hvptwo=&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9007733&hvtargid=pla-306946219383&psc=1), and optionally a USB adapter so you can plug it into your phone.  Then run the script (on a phone you can use Termius or some other terminal app) with the appropriate arguments.

## Cookies

You need to supply a cookie file to the script so it can access the Grades Server.  There are many browser extensions that let you export cookies to a file, any of these will work.  Just login to the grades server, download the cookies, and run the script with that file.  After a period of time, if the script stops working, you may need to do this again.
