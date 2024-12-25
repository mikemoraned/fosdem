import { createModel } from "./model.js";

function bindBookmarks() {
    console.log("Binding Bookmarks");
    // find all bookmark buttons
    const buttons = Array.prototype.slice.call(
        document.querySelectorAll("button.bookmark"),
        0
    );

    // add toggle behavior to each bookmark
    buttons.forEach((el) => {
        const parentEl = el.parentElement.closest("[data-bookmark-status]");
        if (parentEl == null) {
            console.warn("Bookmark button has no parent with data-bookmark-status");
        }
        else {
            el.addEventListener("click", () => {
                // update parent status
                const isBookmarked = parentEl.dataset.bookmarkStatus === "true";
                const newStatus = !isBookmarked;
                parentEl.dataset.bookmarkStatus = newStatus.toString();
            });

            // button is ready to be used
            el.disabled = false;
        }
    });
    console.log("Bookmarks bound");
}

function bindModel(model) {
    // find all events with event-id and bookmark-status
    const stateElements = Array.prototype.slice.call(
        document.querySelectorAll("[data-event-id][data-bookmark-status]"),
        0
    );
    console.log("Found", stateElements.length, "document states");

    // set up an observer which will propagate changes to the model
    const observer = new MutationObserver((mutationsList) => {
        mutationsList.forEach((mutation) => {
          // we assume that we only see changes on 'data-bookmark-status' and that the elemenet has a data-event-id
          const isBookmarked = mutation.target.dataset.bookmarkStatus === "true";
          const eventId = mutation.target.dataset.eventId;
          model.setBookmarkStatus(eventId, isBookmarked);
        });
      });

    const observerOptions = {
        attributes: true, 
        attributeFilter: ['data-bookmark-status'],
    };

    // bind the observer to all state elements
    stateElements.forEach((el) => {
        observer.observe(el, observerOptions);
    });

    // add a listener which is called whenever the model changes for this event
    stateElements.forEach((el) => {
        model.addEventListener(el.dataset.eventId, (status) => {
            el.dataset.bookmarkStatus = status.toString();
        });
    });
}

export async function init() {
    console.log("Initialising bookmarks");
    bindBookmarks();

    const model = await createModel();
    bindModel(model);
}
