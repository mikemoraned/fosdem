# Post 2026 ideas / bugs / niggles

## Cleanups

* [x] remove `connections` feature
    * [x] build:
        * `generate_related`
        * Justfile rules
    * [x] data:
        * `all.limit5.json`
    * [x] web:
        * `main.js`
        * `connections.html`
        * related nav entries
        * home page explanation
        * route handler, `related.rs`
* [ ] remove `next` feature
    * [ ] add redirect from `/next` -> `/{current_year}/timetable/`
    * [ ] 
    * ...
* [ ] inline blog entry:
    * also do data update

## Ideas

* bookmark heatmap of rooms and times (helps identify where/when interesting things are)

## Niggles

* `op` (1Password cli) doesn't work when offline (even though main app does). Error is:
```
[ERROR] 2026/02/03 19:17:20 could not read secret 'op://Dev/fosdem-local-openai-key/password': error initializing client: RequestDelegatedSession: cannot setup session. Please check the logs of the 1Password app.
```

