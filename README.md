# FOSDEM

## What

This repo is behind a [small website](https://fosdem2024.shuttleapp.rs/) where you can find events in
[FOSDEM 2024](https://fosdem.org/2024/) based on content search.

I wrote this because, whilst I really enjoy attending FOSDEM, the amount of possible things to see is large and there is always a chance I can miss something.

The [main website](https://fosdem.org/2024/) does have search, so, _to be totally honest_ I should also say I just wanted to play about with OpenAI Embeddings.

## How

Indexing:

- Event Content is extracted from the Pentabarf version of the Schedule
- The OpenAI Embeddings are looked-up for the title, abstract, and any other relevant content in the Event
- Events and Embeddings info uploaded to Supabase, with Embeddings stored in a `pgvector` column

Lookup:

- Shuttle.rs service used to host website and handle queries
- Search query is converted live to OpenAI Embedding
- Nearest match found via `pgvector` vector distance search
