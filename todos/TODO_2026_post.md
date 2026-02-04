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

## Video

* [x] add simple video player for bookmarks
    * [x] annotate all `a` links in templates which are links to videos with `data-type="video"`
    * client-side in the `/bookmarks/` page:
        * [x] find all video links (using data attribute above)
        * [x] assemble a video which contains all of these, ordered by document order
        * [x] this video should be updated whenever the bookmark status of an event changes, which can happen once on init or afterwards
        * [x] video's should autoplay i.e. advance to next once one ends
* [x] add simple video player for search
    * user-visible behaviour for this is that a [search](https://fosdem.houseofmoran.io/search?q=controversial&limit=20&year=2026):
        * should be expanded to have two [tabs](https://bulma.io/documentation/components/tabs/):
            * one is the current search content, labelled "Events"; this shows all events as now
            * the other is labelled "Videos"; this shows a video player of Events that have videos available
        * the two tabs should only appear if at least one of the Events has a video. If there are none, then only Events content should be shown.
    * this should be implemented by:
        * [x] extracting an askama component for the video player which still allows existing bookmarks video player to work. So:
            * [x] when placed in bookmarks page it has existing behaviour i.e. it finds all events which are bookmarked and collates them into a video
            * [x] when placed on search page, it finds all events which have a video and shows them, regardless of whether they are bookmarked

## Ideas

* bookmark heatmap of rooms and times (helps identify where/when interesting things are)

## Niggles

* `op` (1Password cli) doesn't work when offline (even though main app does). Error is:
```
[ERROR] 2026/02/03 19:17:20 could not read secret 'op://Dev/fosdem-local-openai-key/password': error initializing client: RequestDelegatedSession: cannot setup session. Please check the logs of the 1Password app.
```

