[![Integration-Tests](https://github.com/docspell/dsc/actions/workflows/int_test.yml/badge.svg)](https://github.com/docspell/dsc/actions/workflows/int_test.yml)
[![License](https://img.shields.io/github/license/docspell/dsc.svg?style=flat&color=steelblue)](https://github.com/docspell/dsc/blob/master/LICENSE.txt)
[![Chat](https://img.shields.io/gitter/room/eikek/docspell?style=flat&color=steelblue&logo=gitter)](https://gitter.im/eikek/docspell)

# docspell cli

This is a command line interface to Docspell, a personal document
management system.

The CLI is developed independently with the docspell server. Commands
support the current (at the time of writing) version of docspell. When
the server upgrades its api, the cli is adopted accordingly.

This CLI is meant to be used by humans and other programs. The default
output format is `tabular` which prints a table to stdout. This same
table can be formatted to CSV by using the format option `csv`. These
two formats may omit some details from the server responses for
readability reasons. When using this cli from other programs, or when
you're looking for some detail, the formats `json` and `lisp` are
recommended. They contain every detail in a well structured form.

## State

This CLI is beta … probably forever. I'm using it a lot and hope that
it will be improved over time.

The goal is to eventually have all the [REST
endpoints](https://docspell.org/openapi/docspell-openapi.html) covered
here, or at least those that are frequently used.

## Usage

There are binaries provided at the [release
page](https://github.com/docspell/dsc/releases/latest) that you can
download. Or you can build it as described below.

Run `dsc help` to see a command overview and global options. These
options apply to (almost) all commands. Additionally, each command has
its own set of options and arguments. A command has its help available
via `dsc [subcommand] --help`.


## Config File

The config file is read from the OS defined location, or it can be
specfified explicitly either via an environment variable `DSC_CONFIG`
or as an option. You can run `dsc write-config-file` to create a
default config file in the standard location. The default location on
linux systems is `~/.config/dsc/config.toml`.

The config file looks like this (also, look in the `ci/` folder for
another and always up to date example):

``` toml
docspell_url = "http://localhost:7880"
default_format = "Tabular"
# admin_secret = "test123"
# default_source_id = "<some sorce id>"
# pass_entry = "my/pass/entry"
# default_account = "demo"
pdf_viewer = ["zathura", "{}"]
```

The `pdf_viewer` is used with the `view` command to display the PDF
file. It must be a list where the first element is the program to run
and subsequent elements are its arguments. For each argument, any `{}`
is replaced by the path to the file.


## Authentication

The `login` command can be used to initially login to docspell server.

It accepts a username and password. It also supports the
[pass](https://www.passwordstore.org/) password manager. The user name
can be fixed in the config file as well as the entry to use with
[pass](https://www.passwordstore.org/). This means you can then just
run `dsc login` without any arguments. The retrieved session token is
stored on your file system next to the config file. Subsequent
commands can use the session token. Once it is expired, you need to
call `dsc login` again.

For commands `file-exists` and `upload` it is possible to use a source
id or the integration endpoint instead of being authenticated.


## Building

Install [nix](https://nixos.org/download.html#nix-quick-install) and
run `nix-shell` in the source root. This installs required rust tools.
Alternatively, the rust tool chain can be setup with
[rustup](https://rustup.rs/).

Building the binary for your platform (The second line strips the
binary of debug symbols):

``` bash
> cargo build --release
> strip target/release/dsc
```

This requires the openssl libraries installed on your system.

To build against a statically linked rustls library instead, use:
``` bash
> cargo build --release --no-default-features --features rustls
```

To include a statically linked openssl, build it via:
``` bash
> cargo build --release --no-default-features --features vendored-openssl
```


## Shell Integration

The [library for parsing command line arguments](https://clap.rs/) has
a nice feature that generates completions for various shells. This has
been build into the `dsc` tool. For example, in order to have
completions in [fish](https://fishshell.com/), run:

``` fish
$ dsc generate-completions --shell fish | source
```

… and enjoy tab completion :wink:

Run `dsc generate-completions --help` to see what other shells are
supported.


## Nix Package

The `nix/release.nix` contains a nix expression to build this package.
It can be build using:

``` bash
nix-build nix/ -A dsc
```

The build is updated on each release only; it is not working for the
master branch in general!


## Examples

Reset the password of an account:
``` bash
> dsc admin reset-password user32
┌─────────┬──────────────┬──────────────────┐
│ success │ new password │ message          │
│ true    │ 9rRVrhq19jz  │ Password updated │
└─────────┴──────────────┴──────────────────┘
```


Recreate the full text index:
``` bash
> dsc admin recreate-index
┌─────────┬─────────────────────────────────────┐
│ success │ message                             │
│ true    │ Full-text index will be re-created. │
└─────────┴─────────────────────────────────────┘
```

Search some documents:
``` bash
> dsc search 'date>2020-08-01 corr:acme*'
┌──────────┬─────────────────────────┬───────────┬────────────┬─────┬───────────────┬─────────────┬────────┬──────────────────────────────┬────────┐
│ id       │ name                    │ state     │ date       │ due │ correspondent │ concerning  │ folder │ tags                         │ fields │
│ 7xoiE4Xd │ DOC-20191223-155729.jpg │ created   │ 2020-09-08 │     │ Acme          │             │        │ Invoice                      │        │
│ BV2po65m │ DOC-20200808-154204.jpg │ confirmed │ 2020-08-08 │     │ Acme          │             │        │ Receipt, Tax                 │        │
│ 8GA2ewgE │ DOC-20200807-115654.jpg │ created   │ 2020-08-07 │     │ Acme          │             │        │ Paper, Receipt               │        │
│ FTUnhZ3A │ DOC-20200804-132305.jpg │ confirmed │ 2020-08-04 │     │ Acme          │             │        │ Receipt, Tax                 │        │
│ 6MKV6SEQ │ DOC-20191223-155707.jpg │ confirmed │ 2020-08-03 │     │ Acme          │ Derek Jeter │        │ Important, Information, Todo │        │
└──────────┴─────────────────────────┴───────────┴────────────┴─────┴───────────────┴─────────────┴────────┴──────────────────────────────┴────────┘
```

Use JSON:
``` bash
> dsc -f json search 'date>2020-08-01 corr:acme*' | jq | head -n20
{
  "groups": [
    {
      "name": "2020-09",
      "items": [
        {
          "id": "7xoiE4XdwgD-FTGjD91MptP-yrnKpLrJTfg-Eb2S3BCSd38",
          "name": "DOC-20191223-155729.jpg",
          "state": "created",
          "date": 1599566400000,
          "due_date": null,
          "source": "android",
          "direction": "incoming",
          "corr_org": {
            "id": "GDceAkgrk8m-kjBWUmcuLTV-Zrzp85ByXpX-hq5SS4Yp3Pg",
            "name": "Acme"
          },
          "corr_person": null,
          "conc_person": null,
          "conc_equip": null,
```

Upload some files:
``` bash
> dsc up README.*
File already in Docspell: README.md
Adding to request: README.txt
Sending request …
┌─────────┬──────────────────┐
│ success │ message          │
├─────────┼──────────────────┤
│ true    │ Files submitted. │
└─────────┴──────────────────┘
```


## Making a release

1. Set version in `Cargo.toml`
2. Run `nix-build nix/ -A dsc` and fix hashes
3. Commit + Tag
4. push tag to github

The release is being built by github actions as well as the docker
images.
