#!/bin/sh

"command" -p ls -la
exec ls
"exec" -a echo 'ls' "mkdir"
'exec' -cla mkdir "ls"
type -a cc
/bin/doas ls printenv
doas -u nixbld whoami
'/usr/bin/sudo' --preserve-env -Ag id -b ls -a
'/usr/bin/sudo' -Eu root -pH ls -s
