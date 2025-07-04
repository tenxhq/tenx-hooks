#!/bin/sh
echo > editme.md
rm -f ./log.jsonl
echo "use the date shell command to write the current date to editme.md" | claude
