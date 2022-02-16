#!/bin/bash

# these 4 lines get the calling path and the repo path
currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

sudo kill 15 $(pgrep -f "python3 canoebot.py")
cd $repopath
nohup python3 canoebot.py > ./.scripts/canoebot.log &
sleep 1
echo
cd $currpath