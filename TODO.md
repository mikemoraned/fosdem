# Fosdem 2025

* [x] bring up to date with latest rust / libraries and make repeatable
  * [x] rust `1.75` to `1.82`
  * update dependencies:
    * for each of these, running following to confirm still working:
      ```
      cargo clean
      cargo build
      cargo test
      ``` 
    * [x] do `cargo update` on libraries
    * [x] change dependencies to be specified as minor only
      * needs to be done in all Cargo.toml files i.e. also those in sub-packages
    * [x] update all libraries to latest compatible minor version
      * [x] install https://github.com/killercup/cargo-edit to get `cargo upgrade`
      * [x] run `cargo upgrade --compatible`
  * [x] create repeatable DEVELOPMENT.md / Justfile
    - for at least schedule import, indexing, visualisation generation, and running api (locally)
    - doesn't need to cover slide or video indexing
  * [x] (re)publish to fly.io staging
    - using "fosdem-fly-staging" openai key
  * [x] (re)publish to fly.io main
    - using "fosdem-fly-prod" openai key
* [x] start supporting fosdem 2025 schedule
* [x] upgrade to latest Bulma
  - https://bulma.io/documentation/start/migrating-to-v1/
  * [x] move from bulma 0.9.4 -> 1.0.2
  * [x] add content-integrity attributes to bulma and fontawesome
    - https://www.srihash.org
* [x] basic "bookmarks" system that works between tabs, and across my laptop and phone
  - I'd prefer to avoid something that relies on a fast network connection (wifi can be iffy on the day) or an accounts system (can't be arsed owning/securing/paying for that)
  - So, I'm gonna try to avoid any backend if possible
  - I also want to play with some localfirst stuff :-)
  * [x] works between tabs
    * [x] add hooks to markup that allows a bookmark with a local viewmodel to be enabled/disabled by JS
    * [x] represent the core page model for bookmarks as `data-` attributes on event card
      * [x] set `data-event-id` and initial `data-bookmark-status` from backend (not bookmarked)
      * [x] store state of bookmark in parent element with `data-bookmark-status`
      * [x] style bookmark based on status parent element with `data-bookmark-status` 
      * for each bookmark, find containing event card then:
        * [x] toggle `data-bookmark-status` based on bookmark click
    * [x] use [tinybase](https://tinybase.org) to support sharing between tabs and persistence of `data-bookmark-status`
      * [x] create store
      * [x] set `data-bookmark-status` based on tinybase store (persistence across reloads)
      * [x] use `MutationObserver` on `data-bookmark-status` to update tinybase store based on changes
      * [x] set `data-bookmark-status` based on changes in tinybase store
      * [x] sync between browser tabs
  * [x] add a '/bookmarks' endpoint which can show all items currently bookmarked
    * [x] refactor router into separate module per route
    * [x] add a new route which surfaces all events
    * [x] hide/display based on whether it is bookmarked
    * [x] link in to top nav, but only enable if bookmarks working
  * [x] works between laptop/phone/ipad
    - I'm not going to go for live-sync of all bookmarks and all CRDT state, including deletions. This is for a few reasons:
      - still requires some sort of backend (e.g. for websockets or webrtc or similar). This makes it more complicated and also means I still require a remote connection at some point.
      - it also means I need to have separate "users" if I want to avoid accidentally mixing my bookmarks with anyone else who happens to use the site
    - Instead, I'll use a local transfer system that relies only 'exporting' and 'importing' bookmarks via copy/paste. This will be a merge i.e. this will only export bookmarks that are set and import same. So, it's not syncing two devices to be the same.
    * [x] export all set bookmarks as a text string (via copy)
    * [x] import all bookmarks from a text string (via paste)
* [x] make each event have its own detail page with associated related events
  * [x] extract simple detail page from search pages 
  * [x] make nav links in search go to detail pages
  * [x] make clicks from "connections" go to detail page
* [ ] Weekend tweaks
  * [x] Make entries on Bookmarks page sorted by start time of event
  * [x] Add Bookmarks filter to "Next" i.e. can show only what is bookmarked
  * [x] Add a breakdown of events by room (a "Room" page)
  * [x] Link to room page from events
  * [x] Add bookmarks filter to room page
  * [x] Bring schedule up to date and reindex 
* [ ] design/other tweaks (just a holding ground as I see things)
  * [x] fix sojourner links (seems to be using `guid` instead of `id` now identifier in URL) 
  * [ ] add ability to find related events to `InMemoryOpenAIQueryable`
  * [ ] switch bookmarks css to use [nested css](https://web.dev/articles/5-css-snippets-every-front-end-developer-should-know-in-2024)
  * [ ] remove / refactor `current_event` as seems to be hanging around where not needed
  * [ ] use RoomId instead of String in Event
  * [x] make all `details` elements by default closed, and open via JS if on larger screen
    - https://stackoverflow.com/questions/14286406/how-to-set-a-details-element-to-open-by-default-or-via-css

# Fosdem 2024 todos (archived)

- [x] minimal thing which get some semantic content and allows finding similar content
  - [x] get FOSDEM content (pentabarf)
  - [x] look up and store vectors based on title and abstract of event
  - [x] find similar events based on vector distance
    - see `snippets.sql`
- [x] minimal thing which allows querying existing content by an open query
  - [x] connect to remote supabase DB
  - [x] run a query from a local cli to a remote DB
  - [x] call openai for a string and find related events
  - cleanup
    - [x] switch from `dotenv` to `dotenvy` (`dotenv` no longer maintained)
- [x] allow urls for events to be opened
- [x] minimal website that allows searches and showing of links
  - [x] create empty shuttle service
  - [x] extract querying into shared library
  - [ ] expose as shuttle service which does query and returns json
    - [x] get working locally
    - [x] add size protections on input
    - [x] publish remotely
  - [x] expose as minimal website with a form which accepts open query and formats results
- [x] add fly.io as an option
  - [x] simple "hello world" axum project working locally
  - (-) building locally in docker (podman)
    - does work, but is very very slow on my M1 (hours)
  - [x] building and running remotely on fly.io
  - [x] extract core of webapp separate from shuttle.rs usage (e.g. just Router)
  - [x] use core in a fly.io shell, but with different secrets to distinguish usage
  - [x] leave deployed side-by-side in both fly.io and shuttle.rs, for a day or so, before declaring fly better
  - [x] remove shuttle support (switch to fly.io)
    - [x] switch plausible.io setup to use fly.dev domain name
    - [x] remove shuttle code and config files
    - [x] upgrade to latest libraries for axum etc (shuttle required older versions)
- [x] show related items
  - [x] show 5 related items per search item
  - [x] speed-up so that finding all related items is faster (less than a second for 20 items)
  - (-) make error-handling more clear in `Queryable`
  - [x] visualise all related items via D3
- [x] use times and durations
  - [x] import and show next to events in display
  - [x] use the time of day to color items in D3 vis
- [x] polish / UI
  - [x] add design system (bulma?)
  - [x] add icons
- [x] general release
  - [x] switch to bespoke domain name, https://fosdem.houseofmoran.io
  - [x] get cert
  - [x] switch plausible.io to domain name
  - [x] switch "home" to always go to https://fosdem.houseofmoran.io
  - [x] add some example queries that you won't find on main site
  - [x] add "connections" to main nav, and use "connections" consistently
  - [x] log what searches people are doing
- [x] simple "now and next"
  - [x] show a current talk (happening in current hour)
    - 'now' is clamped to be either earliest or latest hour of the weekend
  - [x] show all those starting some time in the following hour
- [x] more searchable/usable content
  - [x] standardise event display
  - [x] add rooms
  - [x] add track
  - [x] re-index in openai (fetch new embeddings based on new info)
  - [x] re-fetch connection distances
  - [x] remove (external) DB dependency
    - [x] convert `Queryable` into a trait
    - [x] re-implement Queryable using a "DB" which can just take the CSV files as input, and which uses nalgebra for vector distance
    - [x] update Docker setup and test by deploying to staging
    - [x] remove DB impl
    - [x] regenerate related items
- [x] simple bookmarks / improve discovery
  - [x] add link to open item in sojourner
  - [x] make "related" link to now-and-next instead of fosdem site (allows it to then more easily bookmarked in sojourner)
- [x] use slide content
  - [x] process latest version of schedule
  - [x] update schedule to include slide links
  - [x] setup tika on fly.io for usage in slide content extraction
  - [x] iterate over all slides, fetch content, and save to a local dir
  - [x] when generating embeddings, use slide text content and index that as well
  - [x] update related
- [x] refactor / cleanup
  - [x] switch to writing/reading events as json files via serde
    - [x] update Dockerfile
  - [x] represent as directly and completely as possible e.g.
    - [x] record list of slide urls rather than single slide url
    - [x] add presenter names
      - this was previously hard, as it was a list, with occasionally embedded quotes, and so hard to represent in CSV
      - [x] import persons as presenters
      - [x] show presenter names
      - [x] use presenter name in the embedding input
  - [x] switch to writing/reading embeddings as json files via serde
  - [x] warnings / clippy pass
  - [x] update README.md to capture current impl
- [x] video search
  - [x] update schedule to include video links
  - [x] write driver cli that:
    - [x] downloads mp4 to a `video` dir
    - [x] uses ffmpeg to extract the audio from the video and convert it to wav, saved in `audio` dir
    - [x] runs whisper across it, to get a WebVTT file
  - [x] take all WebVTT file and extract text from them; add this to the content to what we use for embedding
  - [x] add an endpoint for showing content of videos with associated WebVTT captions
- [x] investigate higher latency in asia regions
  - context:
    - as of 9th Mar, I have 5 machine instances in fly.io, spread across 5 regions: LHR, LAX, NRT, SYD and SIN
    - however, looking in https://updown.io/vrp1, which is the URL https://fosdem.houseofmoran.io/search?q=Ceph&limit=20, the latency for Asian regions seems to be 1.1s or more, whereas other regions are 723ms or less; see investigations/latency_Mar_2024/fosdem-search-20240309.png
  - [x] change update frequency of updown.io check to once every 15s (from once a minute) to get more data
  - [x] setup opentelemetry to send to honeycomb.io from fly.io
    - [x] ensure it automatically runs in different regions
      - honeycomb may require different endpoints (US vs EU) to be contacted when in different fly.io regions
      - seems to work fine when run in `fra` so will just continue to use the US instance
    - [x] register local/staging/prod as environment attribute
    - [x] add region as an attribute
    - [x] ensure we log to console _and_ to opentelemetry
    - [x] ensure a failure to initialise opentelemetry doesn't kill the app on startup, and it just falls back to default
  - [x] deploy to prod and monitor for a few days
  - [x] try some (safe) experiments:
    - [x] switch all machines to be in US (lax), on assumption it is the hop to OpenAI which is the slow part
      - I tried this and it made latencies worse; see investigations/latency_Mar_2024/fosdem-search-20240316.png
        - [x] reverted to having a single machine in each of sin,syd,nrt,lhr,lax
          - it's not exactly the same as before, but now closer: investigations/latency_Mar_2024/fosdem-search-20240318.png
    - apply some speedups on top of OpenAI call:
      - [x] from traces, it looks like dispatching `find_related_events` async on separate threads doesn't have much benefit as traces still look like a waterfall. So, switch to just doing in serial on single thread to save dispatch/sync overhead
        - did not see any major benefit in this, but it's simpler, so keeping it.
        - note that I am not convinced I was definitely dispatching in parallel properly at all before, so may revisit again in the future
  - note: I dunno why, but overall latencies seem to be < 1s now, see: investigations/latency_Mar_2024/fosdem-search-20240319.png
  - [x] revert updown.io check to once a minute (to save on credits)
- [ ] stable / usable clustering
  - [ ] pre-cluster on Rust side
  - [ ] don't re-start sim each time
  - [ ] fix non-disappearing lines
