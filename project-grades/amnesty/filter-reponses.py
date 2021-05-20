#!/usr/bin/env python3

import sys

if len(sys.argv) < 2:
    sys.stderr.write(f"Usage: {sys.argv[0]} <response csv>")
    sys.exit(1)

responses = open(sys.argv[1]).read().strip().split("\n")[1:]

collected = {}

for response in responses:
    _, _, uid, proj = response.strip().split(",")
    collected[uid] = proj

for uid in collected:
    print(f"{uid},{collected[uid]}")
