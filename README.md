# docspell cli

This is a command line interface to Docspell, a personal document
management system.

## Documentation

The CLI is developed independently with the docspell server. Commands
support the current (at the time of writing) version of docspell. When
the server upgrades its api, the cli is adopted accordingly.


## Config File

The config file is read from the OS defined location, or it can be
specfified explicitly.

## Authentication

The `login` command can be used to initially login to docspell server.

The simple form accepts a username and password. It also supports the
[pass](https://www.passwordstore.org/) password manager. The retrieved
session token is stored on your file system next to the config file.
