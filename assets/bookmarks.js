import { createModel } from "./model.js";

function bindBookmarks() {
    console.log("Binding Bookmarks");
    // find all bookmark buttons
    const buttons = Array.prototype.slice.call(
        document.querySelectorAll("button.bookmark.control"),
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

    // set initial state based on model
    stateElements.forEach((el) => {
        const eventId = el.dataset.eventId;
        const isBookmarked = model.getBookmarkStatus(eventId);
        el.dataset.bookmarkStatus = isBookmarked.toString();
    });

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

function enableBookmarksFeatures() {
    // find all elements related to bookmarks
    const buttons = Array.prototype.slice.call(
        document.querySelectorAll("button.bookmark"),
        0
    );
    const navbarItems = Array.prototype.slice.call(
        document.querySelectorAll("a.navbar-item.bookmark.is-disabled"),
        0
    );

    // enable each element
    buttons.forEach((el) => {
        el.disabled = false;
    });
    navbarItems.forEach((el) => {
        el.classList.remove("is-disabled");
    });
}

export function bindExportImport() {
    const exportShowButton = document.querySelector("button.bookmark#export");
    const exportDialog = document.querySelector("dialog#export-dialog");
    const exportCloseButton = document.querySelector("dialog#export-dialog button.close");

    exportShowButton.addEventListener("click", () => {
        exportDialog.showModal();
    });

    exportCloseButton.addEventListener("click", () => {
        exportDialog.close();
    });
}

export async function init() {
    console.log("Initialising bookmarks");
    bindBookmarks();

    const model = await createModel();
    bindModel(model);

    enableBookmarksFeatures();
}
