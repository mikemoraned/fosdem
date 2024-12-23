import * as AutomergeRepo from "https://esm.sh/@automerge/automerge-repo@2.0.0-alpha.14/slim?bundle-deps"
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
    if (docId != null) {
        console.log("Finding doc with id", docId);
        docHandle = repo.find(docId);
    }
    if (docHandle == null) {
        console.log("Creating new doc as it does not exist");
        docHandle = repo.create(docId, {
            year: 2025,
            bookmarks: []
        });
        console.log("Created new doc with id: ", docHandle.url);
    }
    else {
        console.log("Found existing doc");
    }
    return docHandle;
}

function bindBookmarks() {
    console.log("Binding Bookmarks");
    // find all bookmark buttons
    const $buttons = Array.prototype.slice.call(
        document.querySelectorAll("button.bookmark"),
        0
    );

    // add toggle behavior to each bookmark
    $buttons.forEach((el) => {
        el.addEventListener("click", () => {  
            el.classList.toggle("is-filled");
            el.classList.toggle("is-empty");
        });
    });

    // enable each bookmark
    $buttons.forEach((el) => {
        el.disabled = false;
    });
    console.log("Bookmarks bound");
}

export function init() {
    console.log("Initialising bookmarks");
    const repo = createRepo();
    const docHandle = findOrCreateDoc(repo);
    bindBookmarks();
}
