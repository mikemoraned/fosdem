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

function createModel(originalDoc) {
    const model = (function() {
        let currentDoc = originalDoc;

        function toggleBookmark(eventId) {
            console.log("Toggling bookmark for event", eventId, "in doc", currentDoc.url);
            let updatedDoc = Automerge.change(currentDoc, "toggle-bookmark", d => {
                var entry = d.bookmarks.find(e => e.event_id === eventId);
                if (entry == null) {
                    d.bookmarks.push({ event_id: eventId, bookmarked: true });
                } else {
                    entry.bookmarked = !entry.bookmarked;
                }
            });
            currentDoc = updatedDoc; 
        }

        return {
            toggleBookmark: toggleBookmark,
        };
    })();
    return model;
}


function bindBookmarks(model) {
    console.log("Binding Bookmarks");
    // find all bookmark buttons
    const buttons = Array.prototype.slice.call(
        document.querySelectorAll("button.bookmark"),
        0
    );

    // add toggle behavior to each bookmark
    buttons.forEach((el) => {
        el.addEventListener("click", () => {  
            el.classList.toggle("is-filled");
            el.classList.toggle("is-empty");
            if (el.dataset.eventId) {
                model.toggleBookmark(el.dataset.eventId);
            }
        });
    });

    // enable each bookmark
    buttons.forEach((el) => {
        el.disabled = false;
    });
    console.log("Bookmarks bound");
}

export function init() {
    console.log("Initialising bookmarks");
    const repo = createRepo();
    const doc = findOrCreateDoc(repo);
    const model = createModel(doc);
    bindBookmarks(model);
}
