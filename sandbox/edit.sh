#!/bin/sh
echo > editme.md
rm -f ./log.jsonl
echo "add the word test to editme.md" | claude
