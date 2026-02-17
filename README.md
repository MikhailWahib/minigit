# MiniGit

> A minimal implementation of Git written in Rust.

`minigit` is a small version control system built to understand how Git works under the hood. It implements the core Git object model and provides both high-level "porcelain" commands for daily use and low-level "plumbing" commands for inspecting and manipulating the internal state.

One of the coolest features of `minigit` is **Git Mode** (`--git-mode`), which allows it to operate on existing `.git` repositories!

> NOTE: This project only works on UNIX-based systems and only tested on Linux.

## Features

- **Porcelain Commands**: Easy-to-use commands for standard workflows (`init`, `add`, `commit`, `status`, `log`, `remove`).
- **Plumbing Commands**: Low-level tools to manipulate the object database directly (`hash-object`, `cat-file`, `ls-files`, `update-index`, `write-tree`, `commit-tree`).
- **Interoperability**: Can read and write to real Git repositories using the global `-g` / `--git-mode` flag.

## Installation

You can build and install `minigit` from source using Cargo:

```bash
# Clone the repository
git clone https://github.com/MikhailWahib/minigit.git
cd minigit

# Build and install locally
cargo install --path .
```

Run help command:
```bash
minigit help
```

Or just run it directly:

```bash
cargo run -- help
```

## Usage

### Basic Workflow

Initialize a new repository and make your first commit:

```bash
# Initialize a new .minigit repository
minigit init

# Create a file
echo "Hello, World!" > hello.txt

# Stage the file
minigit add hello.txt

# Check status
minigit status

# Commit changes
minigit commit -m "Initial commit"

# View history
minigit log
```

### Git Mode
This option is global, so you can place it after any subcommand.

Use `--git-mode` (or `-g`) option to make `minigit` operate on `.git` repositories instead of `.minigit`. You can use this for any command!

```bash
# Long option
minigit status --git-mode

# Short option
minigit log -g
```

> Note: This option may not work properly on some repositories because **minigit** doesn't support **packfiles** so far. It's better to try it on small repositories.