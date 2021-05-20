#!/bin/bash

cargo build --release || exit 1

for project in $(env ls p* -d)
do
    continue # XXX: REMOVE THIS LINE TO ACTUALLY DO THE GRADING

    if [ $project == "p5b" ]
    then
        continue
    fi

    echo "---------- Grading all $project submissions ----------"

    # Grade normally
    duedate=$(cat $project/duedate | head -1 | tr -d "\n")

    if [ $project == "p1a" ]
    then
        ../target/release/project-grades -c "115392953" -d "$duedate" -e $project/extensions.csv -l 24,0 -o $project -r ../../rosters/roster-idmap.csv -s $project/submission_metadata.yml
    elif [ $project == "p1b" ]
    then
        ../target/release/project-grades -c "115392953" -d "$duedate" -e $project/extensions.csv -l 24,0 -o $project -r ../../rosters/roster-idmap.csv -s $project/submission_metadata.yml
    elif [ $project == "p2b" ]
    then
        ../target/release/project-grades -c "115392953" -d "$duedate" -e $project/extensions.csv -l 24,0 -o $project -r ../../rosters/roster-idmap.csv -s $project/submission_metadata.yml
    else
        ../target/release/project-grades -c "115392953" -d "$duedate" -e $project/extensions.csv -l 24,0 -o $project -r ../../rosters/roster-idmap.csv -s $project/submission_metadata.yml
    fi
    mv $project/grades.csv $project/grades.normal.csv

    # Grade with due date set to end of semester, no extensions
    ../target/release/project-grades -c "115392953" -d "$duedate" -e $project/extensions.csv -l 9999,0.5 -o $project -r ../../rosters/roster-idmap.csv -s $project/submission_metadata.yml
    mv $project/grades.csv $project/grades.untilend.csv
done

for request in $(cat filtered.csv)
do
    uid=$(echo $request | cut -d, -f1)
    project="p$(echo $request | cut -d, -f2)"

    if [ $project == "p5b" ]
    then
        continue
    fi

    did=$(cat ../../rosters/roster-idmap.csv | grep $uid | cut -d, -f2)
    name=$(cat ../../rosters/roster-idmap.csv | grep $uid | cut -d, -f3)

    echo "------- Applying amnesty to $project for $uid ($did) ($name) --------"

    if [ "$did" == "" ]
    then
        echo -e "\u001b[31mERROR: UID $uid NOT FOUND\u001b[0m"
        continue
    fi

    # Get their normal submission
    normal_wc=$(cat $project/grades.normal.csv | grep "^$did," | wc -l)
    if [ $normal_wc -gt 0 ]
    then
        normal_late=$(cat $project/grades.normal.csv | grep "^$did,\\*" | cut -d, -f3)
        normal_total=$(cat $project/grades.normal.csv | grep "^$did,[^*]" | cut -d, -f3 | awk '{ SUM += $1} END { print SUM }')
        normal_with_penalty=$(python3 -c "import math; print(math.ceil(${normal_total}${normal_late}))")

        echo -e "\u001b[34mNORMAL: Got $normal_total points, penalty $normal_late = $normal_with_penalty\u001b[0m"
    fi

    # Get their later submissions
    late_wc=$(cat $project/grades.untilend.csv | grep "^$did," | wc -l)
    if [ $late_wc -gt 0 ]
    then
        late_late=$(cat $project/grades.untilend.csv | grep "^$did,\\*" | cut -d, -f3)
        late_total=$(cat $project/grades.untilend.csv | grep "^$did,[^*]" | cut -d, -f3 | awk '{ SUM += $1} END { print SUM }')
        late_with_penalty=$(python3 -c "import math; print(math.ceil(${late_total}${late_late}))")

        echo -e "\u001b[34mLATE: Got $late_total points, penalty $late_late = $late_with_penalty\u001b[0m"
    fi

    # Compare
    if [ $normal_wc -eq 0 ]
    then
        if [ $late_wc -eq 0 ]
        then
            # Nothing to do for this student
            echo -e "\u001b[31mNothing to do\u001b[0m"
            continue
        else
            # Take the late one
            best="late"
        fi
    else
        if [ $late_wc -eq 0 ]
        then
            # Take the normal one (i.e., remove the late penalty from their existing submission)
            best="normal"
        elif [ $normal_with_penalty -ge $late_with_penalty ]
        then
            # Take the normal one
            best="normal"
        else
            # Take the late one
            best="late"
        fi
    fi
    
    echo -e "\u001b[32mApplying $best\u001b[0m"
    if [ $best == "normal" ]
    then
        # Use the normal submission
        cat $project/grades.normal.csv | grep "$did," >> $project/grades.csv
    elif [ $best == "late" ]
    then
        # Use the late submission
        cat $project/grades.untilend.csv | grep "$did," >> $project/grades.csv
    fi
done
