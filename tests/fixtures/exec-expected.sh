#!/bin/sh

"command" -p @coreutils@/bin/ls -la
exec @coreutils@/bin/ls
"exec" -a echo @coreutils@/bin/ls "mkdir"
'exec' -cla mkdir @coreutils@/bin/ls
type -a @cc@/bin/cc
doas @coreutils@/bin/ls printenv
doas -u nixbld @coreutils@/bin/whoami
sudo --preserve-env -Ag id -b @coreutils@/bin/ls -a
sudo -Eu root -pH @coreutils@/bin/ls -s
