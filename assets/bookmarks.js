import * as AutomergeRepo from "https://esm.sh/@automerge/automerge-repo@2.0.0-alpha.14/slim?bundle-deps"
import * as Automerge from "https://esm.sh/@automerge/automerge@2.2.8/slim?bundle-deps"
import { IndexedDBStorageAdapter } from "https://esm.sh/@automerge/automerge-repo-storage-indexeddb@2.0.0-alpha.14?bundle-deps"

console.log("Initializing Automerge");
await AutomergeRepo.initializeWasm(fetch("https://esm.sh/@automerge/automerge/dist/automerge.wasm"));
console.log("Automerge initialized");

function createRepo() {
    const repo = new AutomergeRepo.Repo({
        storage: new IndexedDBStorageAdapter(),
        network: [],
    })
    return repo;
}

function findOrCreateDoc(repo) {
    const docId = "automerge:3QyBr3asz8M6LM7GcPcmYTyyXqzH"; // hard-coded for now
    var docHandle = null;
    if (docId != null && AutomergeRepo.isValidAutomergeUrl(docId)) {
        console.log("Finding doc with id", docId);
        docHandle = repo.find(docId);
    }
    else {
        console.log("Creating new doc");
        docHandle = repo.create(docId, {
            year: 2025,
            bookmarks: []
        });
        console.log("Created new doc with id: ", docHandle.url);
    }
    return docHandle;
}

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

class AutomergeModel {
    constructor(doc) {
        this.doc = doc;
    }

    setBookmarkStatus(eventId, status) {
        console.log(`Setting bookmark status for event ${eventId} to ${status}`);
    }
}

function bindAutomerge(model) {
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
}

export function init() {
    console.log("Initialising bookmarks");
    bindBookmarks();

    const repo = createRepo();
    const doc = findOrCreateDoc(repo);
    const model = new AutomergeModel(doc);
    bindAutomerge(model);
}
