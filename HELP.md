# Command-Line Help for `pep257`

This document contains the help content for the `pep257` command-line program.

**Command Overview:**

* [`pep257`↴](#pep257)
* [`pep257 check`↴](#pep257-check)

## `pep257`

A tool to check Rust docstrings against PEP 257 conventions

**Usage:** `pep257 [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `check` — Check a file or directory (defaults to current directory)

###### **Options:**

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity
* `-w`, `--warnings` — Show warnings in addition to errors
* `--format <FORMAT>` — Output format

  Default value: `text`

  Possible values: `text`, `json`

* `--no-fail` — Exit with code 0 even if violations are found



## `pep257 check`

Check a file or directory (defaults to current directory)

**Usage:** `pep257 check [PATH]`

###### **Arguments:**

* `<PATH>` — Path to check (file or directory, defaults to current directory)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

