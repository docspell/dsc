# Integration Tests

For the integration tests, a docspell server (only the restserver) is
started via docker-compose. To start with some data, a new image is
created containing an existing H2 database.

The tests can be run using the script `run-tests.sh` in the source
root.

To play with the tests interactively, run docspell via docker-compose:

``` bash
docker-compose -f docker-compose.yml up -d
```

This builds the image and starts docspell. Then run `dsc` using the
config file from this directory:

``` bash
dsc -c dsc-config.toml source list
```

The server can be stopped via:
``` bash
docker-compose -f docker-compose.yml down
```
