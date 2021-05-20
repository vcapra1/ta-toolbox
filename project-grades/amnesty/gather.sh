#!/bin/bash

BASE_DIR="/mnt/d/Users/Vinnie/Downloads/amnesty-1"

PROJECT_NAMES=(p1a p1b p2a p2b p3 p4a p4b p5a) # note: p5b omitted
PROJECT_IDS=(963955 984600 1009435 1045257 1086846 1140092 1157197 1191264) # note: p5b omitted

for i in "${!PROJECT_NAMES[@]}"
do
    project="${PROJECT_NAMES[i]}"
    id="${PROJECT_IDS[i]}"

    dir=$(fd --base-directory $BASE_DIR -d 2 | grep "assignment_${id}_export$")

    cp "$BASE_DIR/$dir/submission_metadata.yml" "$project/"
done
