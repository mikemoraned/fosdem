# Fosdem 2026 Minimal

These are todos related to bringing things up-to-date without making any major changes in architecture or how it functions.

* [x] bring up to date with latest rust / libraries
  * [x] rust `1.84` to `1.92`
  * [x] update dependencies:
    * [x] for each of these, running following to confirm still working:
      ```
      cargo clean
      cargo build
      cargo test
      ``` 
    * [x] do `cargo update` on libraries
    * [x] update all libraries to latest compatible minor version
      * [x] run `cargo upgrade --compatible`
  * code tidy
    * [x] fix any compiler warnings
    * [x] apply `cargo clippy` lints
* [x] update data for 2025 (fetch latest copy of schedule and re-run all indexing)
* [x] update to a newer debian release (Bookworm)
* [x] integrate with google search console by adding `google-site-verification`
* [x] upgrade to major versions of core libraries
  - note, did not upgrade opentelemetry crates to `0.31` as didn't seem worth it right now given the hassle
* [x] update to use 2026 data
* [ ] support 2025 alongside 2026
  * [x] fetch 2025 and 2026 years
  * [x] import 2025 and 2026 years event data
  * [x] find similarities across all years
  * [x] only generate connections for current year
  * [x] show year of event in title
  * [x] change all links to be prefixed by year e.g. `/event/6197/` becomes `/2025/event/6197/`
  * [ ] any previous links to go to the 2025 version e.g `/event/6197/` returns same content as `/2025/event/6197/`
  * [ ] search across years:
    * [x] update indexing to consume data from all schedules for 2025 and 2026
    * [ ] add a filter that allows restriction by year
  * [ ] when importing bookmarks, any ids that are just a single number get mapped to `2025-number`
* [ ] add `sitemap.xml`
    - https://crates.io/crates/sitemap_generator ?