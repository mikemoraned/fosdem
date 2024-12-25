import { createMergeableStore } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/+esm';

export class Model {
    constructor() {
        this.store = createMergeableStore('fosdem2025');
    }

    setBookmarkStatus(eventId, status) {
        console.log(`Setting bookmark status for event ${eventId} to ${status}`);
        this.store.setValue(eventId, status);
        console.log(this.store.getContent());
        console.log(this.store.getMergeableContent());
    }
}