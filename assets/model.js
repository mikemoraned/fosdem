import { createStore } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/+esm';

export class Model {
    constructor() {
    }

    setBookmarkStatus(eventId, status) {
        console.log(`Setting bookmark status for event ${eventId} to ${status}`);
    }
}