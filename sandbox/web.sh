#!/bin/sh
echo > editme.md
rm -f ./log.jsonl
echo "please go look at what the top story is on hackernews, and write the title to editme.md" | claude --debug --model sonnet
