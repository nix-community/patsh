#!/bin/sh

command -p @coreutils@/bin/ls mkdir -v
"command" -pv @coreutils@/bin/ls @coreutils@/bin/mkdir
exec @coreutils@/bin/ls
"exec" -a echo @coreutils@/bin/ls mkdir
'exec' -cla mkdir @coreutils@/bin/ls
type -a @cc@/bin/cc @coreutils@/bin/ls
doas @coreutils@/bin/ls printenv
doas -u nixbld @coreutils@/bin/whoami
sudo --preserve-env -gA @coreutils@/bin/id ls -a
sudo -Eu id -pH @coreutils@/bin/ls -s
