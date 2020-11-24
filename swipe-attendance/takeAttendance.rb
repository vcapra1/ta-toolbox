#!/usr/bin/ruby

require 'colored'

if ARGV.length < 4
    STDERR.puts "Usage: takeAttendance.rb <courseID> <assignmentID> <cookies> <logFile>"
    exit 1
end

$URL = "https://grades.cs.umd.edu/classWeb/takeAttendance.cgi"

courseID = ARGV[0]
assignmentID = ARGV[1]

$REFERER = "#{$URL}?assignmentID=#{assignmentID}&courseID=#{courseID}"

cookies = ARGV[2]
logFile = ARGV[3]

open(logFile, 'a') do |f|
    f.puts "----------------------------"
    f.puts Time.new.inspect
    f.puts
end

def success_bell
    #STDERR.puts "\x07"
end

def error_bell
    1.times do
        STDERR.print "\x07"
	STDERR.flush
	sleep 0.15
    end
end

loop do
    print "Swipe Card Now (or enter UID): ".white

    line = STDIN.gets
    if line.nil? then
        puts
        break
    end
    line.strip!
    if line == "" then
        open(logFile, 'a') do |f|
            f.puts
        end

        next
    elsif line == "exit" or line == "quit" or line == "q" then
        break
    elsif line =~ /^[0-9]{9}$/
        # Is UID
        uid = line
        idcard = ""
    else
        # Is Swipe
        uid = ""
        idcard = line
    end

    idcard.gsub!(";", "%3B")
    idcard.gsub!("?", "%3F")

    # Make request
    data = "courseID=#{courseID}&assignmentID=#{assignmentID}&uid=#{uid}&idcard=#{idcard}&submit=Take+Attendance"

    result = `curl -X POST --cookie '#{cookies}' -H 'Content-Type: application/x-www-form-urlencoded' -H 'Referer: https://grades.cs.umd.edu/classWeb/takeAttendance.cgi?assignmentID=#{assignmentID}&courseID=#{courseID}' -H 'Host: grades.cs.umd.edu' 'https://grades.cs.umd.edu/classWeb/takeAttendance.cgi' --data 'courseID=#{courseID}&assignmentID=#{assignmentID}&uid=#{uid}&idcard=#{idcard}&submit=Take+Attendance' -v 2> /dev/null`

    if result =~ /attendeeID=(\d+)/ then
        stuID = $1

        info = `curl --cookie '#{cookies}' -H 'Referer: https://grades.cs.umd.edu/classWeb/viewComments.cgi' -H 'Host: grades.cs.umd.edu' 'https://grades.cs.umd.edu/classWeb/viewComments.cgi?stuID=#{stuID}&courseID=#{courseID}' 2> /dev/null | grep "<h2>"`

        if info =~ /<h2>(.*)<\/h2>/
            name = $1.strip
        else
            STDERR.puts "ERROR".red.bold
	    error_bell
            next
        end

        if info =~ /UID Number: (\d{9})/
            uid = $1
        else
            STDERR.puts "ERROR".red.bold
	    error_bell
            next
        end

        puts "Success: #{name.bold}".green
	success_bell

        open(logFile, 'a') do |f|
            f.puts "#{uid},\"#{name}\""
        end
    else
        STDERR.puts "ERROR (probably an expired session)".red.bold
	error_bell
    end
end

puts "Exiting..."
