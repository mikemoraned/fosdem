# Post 2026 ideas / bugs / niggles

(Some of these are TODO's for me, and some are specifically for claude)

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
* [x] remove `next` feature
    * [x] web:
        * remove nav links for `/next`
        * remove `now_and_next.html`
    * [x] add a redirect from `/next` -> `/{current_year}/timetable/` (so people get taken to current years timetable)
        * do this by removing `next.rs` handler and adding a `next` handler to `timetable.rs`
    * [x] find and remove unneeded code like `find_next_events` or anything else that was previously used by `next.rs` and not by anything else
* [x] add timetable to nav:
    * update nav to have a timetable link which goes to current year
    * choose an appropriate icon (`fa-calendar-days`)
    * use `is-hidden-mobile` on text labels so icons only show on mobile, icon+text on tablet/desktop
* [x] blog entry on changes

## Ideas

* bookmark heatmap of rooms and times (helps identify where/when interesting things are)

## Niggles

* `op` (1Password cli) doesn't work when offline (even though main app does). Error is:
```
[ERROR] 2026/02/03 19:17:20 could not read secret 'op://Dev/fosdem-local-openai-key/password': error initializing client: RequestDelegatedSession: cannot setup session. Please check the logs of the 1Password app.
```

