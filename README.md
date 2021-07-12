# docspell cli

This is a command line interface to Docspell, a personal document
management system.

The CLI is developed independently with the docspell server. Commands
support the current (at the time of writing) version of docspell. When
the server upgrades its api, the cli is adopted accordingly.

## Usage

Run `dsc --help` to see a command overview. There are common options
that apply to (almost) all commands and each command has its own set
of options and arguments.


## Config File

The config file is read from the OS defined location, or it can be
specfified explicitly. You can run `dsc write-config-file` to create a
default config file in the standard location.

## Authentication

The `login` command can be used to initially login to docspell server.

The simple form accepts a username and password. It also supports the
[pass](https://www.passwordstore.org/) password manager. The retrieved
session token is stored on your file system next to the config file.
Subsequent commands can use the session token. Once it is expired, you
need to call `dsc login` again.
