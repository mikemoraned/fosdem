// This approach requires the initializeWasm export not yet found in the stable 1.2.1 release.
import * as AutomergeRepo from "https://esm.sh/@automerge/automerge-repo@2.0.0-alpha.14/slim?bundle-deps"
import { IndexedDBStorageAdapter } from "https://esm.sh/@automerge/automerge-repo-storage-indexeddb@2.0.0-alpha.14?bundle-deps"
import { BrowserWebSocketClientAdapter } from "https://esm.sh/@automerge/automerge-repo-network-websocket@2.0.0-alpha.14?bundle-deps"
import { MessageChannelNetworkAdapter } from "https://esm.sh/@automerge/automerge-repo-network-messagechannel@2.0.0-alpha.14?bundle-deps"

console.log("Initializing Automerge");
await AutomergeRepo.initializeWasm(fetch("https://esm.sh/@automerge/automerge/dist/automerge.wasm"));
console.log("Automerge initialized");

export function bindBookmarks() {
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