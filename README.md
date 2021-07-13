[![Integration-Tests](https://github.com/docspell/dsc/actions/workflows/int_test.yml/badge.svg)](https://github.com/docspell/dsc/actions/workflows/int_test.yml)


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

## Usage

Run `dsc --help` to see a command overview. There are common options
that apply to (almost) all commands and each command has its own set
of options and arguments.


## Config File

The config file is read from the OS defined location, or it can be
specfified explicitly. You can run `dsc write-config-file` to create a
default config file in the standard location. The default location on
linux systems is `~/.config/dsc/config.toml`.

The config file looks like this:

``` toml
docspell_url = "http://localhost:7880"
default_format = "Tabular"
admin_secret = "test123"
default_source_id = "<some sorce id>"
pass_entry = "my/pass/entry"
default_account = "demo"
```


## Authentication

The `login` command can be used to initially login to docspell server.

It accepts a username and password. It also supports the
[pass](https://www.passwordstore.org/) password manager. The user name
can be fixed in the config file as well as the entry to use with
[pass](https://www.passwordstore.org/) which means you can then just
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


## Examples

Reset the password of an account:
``` bash
> dsc admin reset-password user32
Docspell at: http://localhost:7880
┌─────────┬──────────────┬──────────────────┐
│ success │ new password │ message          │
│ true    │ 9rRVrhq19jz  │ Password updated │
└─────────┴──────────────┴──────────────────┘
```


Recreate the full text index:
``` bash
> dsc admin recreate-index
Docspell at: http://localhost:7880
┌─────────┬─────────────────────────────────────┐
│ success │ message                             │
│ true    │ Full-text index will be re-created. │
└─────────┴─────────────────────────────────────┘
```

Search some documents:
``` bash
> dsc search --with-details 'date>2020-08-01 corr:acme*'
Docspell at: http://localhost:7880
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
> dsc -f json search --with-details 'date>2020-08-01 corr:acme*' | jq | head -n20
Docspell at: https://docs.daheim.site
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
