`# Post 2026 ideas / bugs / niggles

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
* [x] video improvements:
    * [x] in player, make title be a link to the event URL of the playing video
    * [x] add ability to select webm video as preferred video type, but don't use it yet
        * [x] modelled after `mp4_video_link` add `webm_video_link` which finds `.webm`
            * ensure tests added; if real examples needed, see `events.json` for current year (`2026`)
            * once tests passing, refactor to extract any shared code for finding video links
        * [x] add `video_link` which returns a generic link and chooses `mp4` if it is available, and falls back to `webm` if not
        * [x] update any calls of `mp4_video_link`:
            * [x] where it doesn't actually care about it being an mp4 video and just cares about a video existing, to use `has_video`
            * [x] where it just wants a link, to use `video_link`
    * [x] support specifying multiple video formats and let the browser decide which it can show
        * [x] refactor: follow the NewType pattern and make `mp4_video_link` and `webm_video_link` return a struct; update usages appropriately
        * [x] add a `video_links` which returns a list of these NewTypes. add appropriate tests following existing semantics
        * [x] update the askama event templates to have an inline <video> element section, wrapped in a details element which is closed by default, which contains multiple <source> elements, one for each type of video returned by `video_links`; if no videos are available then no section should exist
        * [x] update video_player.js so that loadVideo uses the list of sources derived from above data i.e. so that multiple sources are listed if multiple formats available
        * [x] update the event template so that the video link in the header which indicates it has a video is just a piece of text, not a link
        * [x] favour webm over mp4 by returning that first in the list from `video_links`
        * [x] remove `video_link` if it is no longer used
        * [x] update video link parsing so that:
            * [x] there is a new `codecs` method that returns an optional string
                * [x] for mp4 this is None
                * [x] for webm it is Some("av01.0.08M.08.0.110.01.01.01.0")
                    * this is taken from main fosdem.org website
        * [x] update askama templates so that when a VideoLink enum has Some `codec` then it is added as an attribute
        * [x] update video link JS so that any `codec` attributes are copied over
    * [ ] simplify video player by using the built-in controls, so no need for my own play/pause buttons

## Ideas

* bookmark heatmap of rooms and times (helps identify where/when interesting things are)

## Niggles

* `op` (1Password cli) doesn't work when offline (even though main app does). Error is:
```
[ERROR] 2026/02/03 19:17:20 could not read secret 'op://Dev/fosdem-local-openai-key/password': error initializing client: RequestDelegatedSession: cannot setup session. Please check the logs of the 1Password app.
```

