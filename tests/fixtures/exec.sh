#!/bin/sh

command -p ls mkdir -v
"command" -pv ls "/bin/mkdir"
exec ls
"exec" -a echo 'ls' mkdir
'exec' -cla mkdir "ls"
type -a cc ls
/bin/doas ls printenv
doas -u nixbld whoami
'/usr/bin/sudo' --preserve-env -gA id ls -a
'/usr/bin/sudo' -Eu id -pH ls -s
