# MiniGit

> A minimal implementation of Git written in Rust.

MiniGit is a small version control system built to understand how Git works under the hood. It has porcelain commands (`init`, `add`, `commit`, `status`, `log`, `remove`) and plumbing commands (`hash-object`, `cat-file`, `ls-files`, `update-index`, `write-tree`, `commit-tree`).

It also supports **Git Mode** (`-g` / `--git-mode`) to operate on `.git` repositories.

> NOTE: This project works on UNIX-based systems and is tested on Linux.

## Installation

```bash
git clone https://github.com/MikhailWahib/minigit.git
cd minigit
cargo install --path .
```

Or run directly:

```bash
cargo run -- --help
```

## Usage

### Command list (`minigit --help`)

```bash
$ minigit --help
Usage: minigit [OPTIONS] <COMMAND>

Commands:
  init          Initialize a new Minigit repository
  add           Add file contents to the index
  commit        Record changes to the repository
  status        Show the working tree status
  log           Show commit logs
  remove        Unstage file(s) - equivalent to `git restore --staged`
  hash-object   Compute the hash of an object
  cat-file      Print the contents of a file in the Minigit database
  ls-files      Print the index file
  update-index  Update the index
  write-tree    Create a tree object from the current index
  commit-tree   Create a new commit object from the specified tree
  help          Print this message or the help of the given subcommand(s)

Options:
  -g, --git-mode  Use .git as a root dir instead of .minigit; Used to test on real git repos
  -h, --help      Print help
  -V, --version   Print version
```

### Basic operations in `.minigit`

```bash
$ minigit init
repo initialized at /tmp/minigit-readme-demo/native/.minigit

$ printf 'Hello, World!\n' > hello.txt

$ minigit add hello.txt

$ minigit status
Changes to be committed:
  (use "minigit remove <file>..." to unstage)

	new file:   hello.txt

$ minigit commit -m 'Initial commit'

$ minigit log
commit 6c891b44577fc5ed29316bf9a5429c35c468962f
Author: Mikhail <mikhailwahib1@gmail.com>
Date:   Wed Feb 18 07:23:17 2026 +0200

    Initial commit
```

### Operate on `.git` using Git Mode

```bash
$ git init
Initialized empty Git repository in /tmp/minigit-readme-demo/gitmode/.git/

$ printf 'from git mode\n' > note.txt

$ minigit add note.txt -g

$ minigit status -g
Changes to be committed:
  (use "minigit remove <file>..." to unstage)

	new file:   note.txt

$ minigit commit -m 'Commit via minigit in git mode' -g

$ git log --oneline -n 1
4feefcc Commit via minigit in git mode
```

## Notes

- Packfiles are not supported yet, so Git mode works best on small/fresh repos.
- MiniGit uses existing Git global config for now, so you should have Git installed and configured on your machine.
