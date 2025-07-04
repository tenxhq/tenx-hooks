#!/bin/sh
echo > editme.md
rm -f ./log.jsonl
echo "using two agents in parellel, add the word test to editme.md and editme2.md" | claude --debug --model sonnet
