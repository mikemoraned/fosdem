# Install

Most stuff used here is defined in `./backend/Justfile`, so you'll need to install `just`

## OpenAI keys

To create openai keys, go https://platform.openai.com/api-keys and select the `fosdem` project.

These are different service-accounts:
* `fosdem-local`: used for local dev
* fly.io: used for respective environments:
    * `fosdem-fly-staging`
    * `fosdem-fly-prod`

These are remembered in 1Password under `OpenAI Fosdem {{service-account}} Key` e.g. `OpenAI Fosdem fosdem-local Key`

For local dev key is stored in a `.env` file as `OPENAI_API_KEY` for access by the programs. For usage in `fly.io` the same password needs to be stored in secrets.


# Importing Schedule

```
just import_schedule
```


# Indexing

## Next

```
RUST_LOG=info just index_next
```

(Can also set `RUST_LOG=debug` for more verbose info)

## Past

TODO

