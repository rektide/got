# got

> `got` is a tool/toolkit for doing the git things your past self should have done

# toolkit

| tool | use |
| --- | --- |
| `goldest` bin/executable | find the oldest changes and get a datestamp for that file |
| `git commitd` git alias | `commit` using the most recent date of what is staged |
| `git statusd` git alias | `status` but focused on modified time |
| `hook-got` shell hook | shell hook to build pertinent environment variables |
| `gotsel` micro TUI | git staging selection tree tool |

## `goldest`

> A small executable for identifying the oldest file

```sh
$ goldest
README.md 01-15-26T16:20:00Z
```

| **option** | description |
| --- | --- |
| `-f` | return only file |
| `-d` | return only date |
| `-u[filter]` | filter equivalent to git status -u filtering |
| `--lines/-l [num-of-lines]` | number of results to show |
| `--skip/-S [skip]` | skip `s` results |
| `-s/--short` | git status --short output format |
| `--porcelain` | git status --porcelain output format |

## `hook-got`

```sh
$ echo $GOT_F : $GOT_D
README.md : 01-15-26T16:20:00Z
```

## `commitd`

> Git alias to enrich `commit` that uses the most recent file modified date of the staged files

```sh
$ git commitd -m'make it so'
$ git log HEAD --date=iso8601-strict

commit c0dec0dec0de
Author: rektide
Date: 1-15-26T16:20:00Z
```

## `statusd`

> Git alias to enrich `git status`, without other decorations, sorted by & showing time

(Actually a wrapper / mode of `goldest`)

```sh

```

## `gotsel`

The human omni-tool for staging a commit, with by default a date-of-modification driven (fancy word for "sorted") UI

```sh
$ gotsel
<nice fancy tree of changes you can cursor (curse? lol) though>
```

# bonus

## nah

Nah is a subfeature of got, that allows for ignoring files.

```sh
$ git status -u # insert -u flag looking for things that are uncommited or changed
.scratchfile
$ nah add .scratchfile

```

##
