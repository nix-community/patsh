#!/bin/sh

command -p @coreutils@/bin/ls mkdir -v
"command" -pv @coreutils@/bin/mkdir echo
exec @coreutils@/bin/ls
"exec" -a ls @coreutils@/bin/echo mkdir
'exec' -cla mkdir @coreutils@/bin/ls
type echo
"type" -a @cc@/bin/cc @coreutils@/bin/ls
doas @coreutils@/bin/echo printenv
doas -u nixbld @coreutils@/bin/whoami
sudo --preserve-env -gA @coreutils@/bin/printf ls -a
sudo -Eu id -pH @coreutils@/bin/ls -s
