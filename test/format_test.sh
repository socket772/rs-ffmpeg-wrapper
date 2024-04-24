#!/bin/bash
array=("mp3" "m4a" "flac" "ogg" "wav" "aac" "m4b" "oga" "opus" "webm")
for i in ${array[@]};
do
    cargo run -- -t $(nproc) -i $1 -o $2 -f $i
done
