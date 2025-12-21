import { createMergeableStore } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/+esm';
import { createBroadcastChannelSynchronizer } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/synchronizers/synchronizer-broadcast-channel/+esm';
import { createLocalPersister } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/persisters/persister-browser/+esm';

export async function createModel() {
    const store = createMergeableStore('fosdem2026');

    const persister = createLocalPersister(store, 'fosdem2026');
    await persister.load();
    await persister.startAutoSave()

    const synchronizer = createBroadcastChannelSynchronizer(
        store,
        'fosdem2026SyncChannel',
        () => {
            console.log('sent a message');
        },
        () => {
            console.log('received a message');
        }
    );
    synchronizer.addStatusListener((synchronizer, status) => {
        console.log(
            `${synchronizer.getChannelName()} channel status changed to ${status}`,
        );
    });
    await synchronizer.startSync();
    return new Model(store);
}

class Model {
    constructor(store) {
        this.store = store
    }

    getBookmarkStatus(eventId) {
        return this.store.getValue(eventId) || false;
    }

    setBookmarkStatus(eventId, status) {
        console.log(`Setting bookmark status for event ${eventId} to ${status}`);
        this.store.setValue(eventId, status);
        console.log(this.store.getContent());
        console.log(this.store.getMergeableContent());
    }

    addEventListener(eventId, listenerFn) {
        this.store.addValuesListener((store, getValueChange) => {
            const [hasChanged, _oldValue, newValue] = getValueChange(eventId);
            if (hasChanged) {
                console.log(`Event ${eventId} changed to ${newValue}`);
                listenerFn(newValue);
            }
        }, false);
    }

    exportEventIdsAsText() {
        const eventIds = [];
        this.store.forEachValue((eventId, isBookmarked) => {
            if (isBookmarked) {
                eventIds.push(eventId);
            }
        });
        return eventIds.join(' ');
    }

    importEventIdsFromText(text) {
        const possibleEventIds = text.split(' ');
        possibleEventIds.forEach((possibleEventId) => {
            if (possibleEventId.match(/\d+/)) {
                this.store.setValue(`2025-${possibleEventId}`, true);
            }
            if (possibleEventId.match(/\d+-\d+/)) {
                this.store.setValue(possibleEventId, true);
            }
        });
    }
}