# Fosdem 2025

* (/) bring up to date with latest rust / libraries and make repeatable
  * (/) rust `1.75` to `1.82`
  * update dependencies:
    * for each of these, running following to confirm still working:
      ```
      cargo clean
      cargo build
      cargo test
      ``` 
    * (/) do `cargo update` on libraries
    * (/) change dependencies to be specified as minor only
      * needs to be done in all Cargo.toml files i.e. also those in sub-packages
    * (/) update all libraries to latest compatible minor version
      * (/) install https://github.com/killercup/cargo-edit to get `cargo upgrade`
      * (/) run `cargo upgrade --compatible`
  * (/) create repeatable DEVELOPMENT.md / Justfile
    - for at least schedule import, indexing, visualisation generation, and running api (locally)
    - doesn't need to cover slide or video indexing
  * (/) (re)publish to fly.io staging
    - using "fosdem-fly-staging" openai key
  * (/) (re)publish to fly.io main
    - using "fosdem-fly-prod" openai key
* (x) switch staging to be supporting fosdem 2025 schedule
  * (x) ...

# Fosdem 2024 todos (archived)

- (/) minimal thing which get some semantic content and allows finding similar content
  - (/) get FOSDEM content (pentabarf)
  - (/) look up and store vectors based on title and abstract of event
  - (/) find similar events based on vector distance
    - see `snippets.sql`
- (/) minimal thing which allows querying existing content by an open query
  - (/) connect to remote supabase DB
  - (/) run a query from a local cli to a remote DB
  - (/) call openai for a string and find related events
  - cleanup
    - (/) switch from `dotenv` to `dotenvy` (`dotenv` no longer maintained)
- (/) allow urls for events to be opened
- (/) minimal website that allows searches and showing of links
  - (/) create empty shuttle service
  - (/) extract querying into shared library
  - (x) expose as shuttle service which does query and returns json
    - (/) get working locally
    - (/) add size protections on input
    - (/) publish remotely
  - (/) expose as minimal website with a form which accepts open query and formats results
- (/) add fly.io as an option
  - (/) simple "hello world" axum project working locally
  - (-) building locally in docker (podman)
    - does work, but is very very slow on my M1 (hours)
  - (/) building and running remotely on fly.io
  - (/) extract core of webapp separate from shuttle.rs usage (e.g. just Router)
  - (/) use core in a fly.io shell, but with different secrets to distinguish usage
  - (/) leave deployed side-by-side in both fly.io and shuttle.rs, for a day or so, before declaring fly better
  - (/) remove shuttle support (switch to fly.io)
    - (/) switch plausible.io setup to use fly.dev domain name
    - (/) remove shuttle code and config files
    - (/) upgrade to latest libraries for axum etc (shuttle required older versions)
- (/) show related items
  - (/) show 5 related items per search item
  - (/) speed-up so that finding all related items is faster (less than a second for 20 items)
  - (-) make error-handling more clear in `Queryable`
  - (/) visualise all related items via D3
- (/) use times and durations
  - (/) import and show next to events in display
  - (/) use the time of day to color items in D3 vis
- (/) polish / UI
  - (/) add design system (bulma?)
  - (/) add icons
- (/) general release
  - (/) switch to bespoke domain name, https://fosdem.houseofmoran.io
  - (/) get cert
  - (/) switch plausible.io to domain name
  - (/) switch "home" to always go to https://fosdem.houseofmoran.io
  - (/) add some example queries that you won't find on main site
  - (/) add "connections" to main nav, and use "connections" consistently
  - (/) log what searches people are doing
- (/) simple "now and next"
  - (/) show a current talk (happening in current hour)
    - 'now' is clamped to be either earliest or latest hour of the weekend
  - (/) show all those starting some time in the following hour
- (/) more searchable/usable content
  - (/) standardise event display
  - (/) add rooms
  - (/) add track
  - (/) re-index in openai (fetch new embeddings based on new info)
  - (/) re-fetch connection distances
  - (/) remove (external) DB dependency
    - (/) convert `Queryable` into a trait
    - (/) re-implement Queryable using a "DB" which can just take the CSV files as input, and which uses nalgebra for vector distance
    - (/) update Docker setup and test by deploying to staging
    - (/) remove DB impl
    - (/) regenerate related items
- (/) simple bookmarks / improve discovery
  - (/) add link to open item in sojourner
  - (/) make "related" link to now-and-next instead of fosdem site (allows it to then more easily bookmarked in sojourner)
- (/) use slide content
  - (/) process latest version of schedule
  - (/) update schedule to include slide links
  - (/) setup tika on fly.io for usage in slide content extraction
  - (/) iterate over all slides, fetch content, and save to a local dir
  - (/) when generating embeddings, use slide text content and index that as well
  - (/) update related
- (/) refactor / cleanup
  - (/) switch to writing/reading events as json files via serde
    - (/) update Dockerfile
  - (/) represent as directly and completely as possible e.g.
    - (/) record list of slide urls rather than single slide url
    - (/) add presenter names
      - this was previously hard, as it was a list, with occasionally embedded quotes, and so hard to represent in CSV
      - (/) import persons as presenters
      - (/) show presenter names
      - (/) use presenter name in the embedding input
  - (/) switch to writing/reading embeddings as json files via serde
  - (/) warnings / clippy pass
  - (/) update README.md to capture current impl
- (/) video search
  - (/) update schedule to include video links
  - (/) write driver cli that:
    - (/) downloads mp4 to a `video` dir
    - (/) uses ffmpeg to extract the audio from the video and convert it to wav, saved in `audio` dir
    - (/) runs whisper across it, to get a WebVTT file
  - (/) take all WebVTT file and extract text from them; add this to the content to what we use for embedding
  - (/) add an endpoint for showing content of videos with associated WebVTT captions
- (x) investigate higher latency in asia regions
  - context:
    - as of 9th Mar, I have 5 machine instances in fly.io, spread across 5 regions: LHR, LAX, NRT, SYD and SIN
    - however, looking in https://updown.io/vrp1, which is the URL https://fosdem.houseofmoran.io/search?q=Ceph&limit=20, the latency for Asian regions seems to be 1.1s or more, whereas other regions are 723ms or less; see investigations/latency_Mar_2024/fosdem-search-20240309.png
  - (/) change update frequency of updown.io check to once every 15s (from once a minute) to get more data
  - (/) setup opentelemetry to send to honeycomb.io from fly.io
    - (/) ensure it automatically runs in different regions
      - honeycomb may require different endpoints (US vs EU) to be contacted when in different fly.io regions
      - seems to work fine when run in `fra` so will just continue to use the US instance
    - (/) register local/staging/prod as environment attribute
    - (/) add region as an attribute
    - (/) ensure we log to console _and_ to opentelemetry
    - (/) ensure a failure to initialise opentelemetry doesn't kill the app on startup, and it just falls back to default
  - (/) deploy to prod and monitor for a few days
  - (/) try some (safe) experiments:
    - (/) switch all machines to be in US (lax), on assumption it is the hop to OpenAI which is the slow part
      - I tried this and it made latencies worse; see investigations/latency_Mar_2024/fosdem-search-20240316.png
        - (/) reverted to having a single machine in each of sin,syd,nrt,lhr,lax
          - it's not exactly the same as before, but now closer: investigations/latency_Mar_2024/fosdem-search-20240318.png
    - apply some speedups on top of OpenAI call:
      - (/) from traces, it looks like dispatching `find_related_events` async on separate threads doesn't have much benefit as traces still look like a waterfall. So, switch to just doing in serial on single thread to save dispatch/sync overhead
        - did not see any major benefit in this, but it's simpler, so keeping it.
        - note that I am not convinced I was definitely dispatching in parallel properly at all before, so may revisit again in the future
  - note: I dunno why, but overall latencies seem to be < 1s now, see: investigations/latency_Mar_2024/fosdem-search-20240319.png
  - (/) revert updown.io check to once a minute (to save on credits)
- (x) stable / usable clustering
  - (x) pre-cluster on Rust side
  - (x) don't re-start sim each time
  - (x) fix non-disappearing lines
