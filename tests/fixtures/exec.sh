#!/bin/sh

command -p ls mkdir -v
"command" -pv "/bin/mkdir" echo
exec ls
"exec" -a ls 'echo' mkdir
'exec' -cla mkdir "ls"
type echo
"type" -a cc ls
/bin/doas echo printenv
doas -u nixbld whoami
sudo --preserve-env -gA printf ls -a
'/usr/bin/sudo' -Eu id -pH ls -s
