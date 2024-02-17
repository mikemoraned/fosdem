# FOSDEM

## What

This repo is behind a [small website](https://fosdem.houseofmoran.io/) where you can find events in
[FOSDEM 2024](https://fosdem.org/2024/) based on content search or via [connections](https://fosdem.houseofmoran.io/connections/).

I wrote this because, whilst I really enjoy attending FOSDEM, the amount of possible things to see is large and there is always a chance I can miss something.

The [main website](https://fosdem.org/2024/) does have search, so, _to be totally honest_ I should also say I just wanted to play about with OpenAI Embeddings. However, one outcome is that you'll find results for queries here you won't find on main site e.g. ["controversial"](https://fosdem.houseofmoran.io/search?q=controversial&limit=20")

## How

Indexing:

- Event Content is extracted from the Pentabarf version of the Schedule
- The OpenAI Embeddings are looked-up for the title, abstract, and any other relevant content in the Event info we can derive from the Schedule. Additionally:

  - if slides are available, these are downloaded, text is extracted using Apache Tika Server, and these are added to the input to the embedding

- Events and Embeddings info are saved in JSON files; these are the `index`

Lookup:

- fly.io service used to host website and handle queries
- JSON index files are loaded into memory on startup
- For each query:
  - Search query is converted live to an OpenAI Embedding
  - Nearest match found via vector distance search, using `nalgebra` for distance calculation
