import { createMergeableStore } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/+esm';
import { createBroadcastChannelSynchronizer } from 'https://cdn.jsdelivr.net/npm/tinybase@5.4.4/synchronizers/synchronizer-broadcast-channel/+esm';


export async function createModel() {
    const store = createMergeableStore('fosdem2025');
    const synchronizer = createBroadcastChannelSynchronizer(
        store,
        'fosdem2025SyncChannel',
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

    setBookmarkStatus(eventId, status) {
        console.log(`Setting bookmark status for event ${eventId} to ${status}`);
        this.store.setValue(eventId, status);
        console.log(this.store.getContent());
        console.log(this.store.getMergeableContent());
    }

    addEventListener(eventId, listenerFn) {
        this.store.addValuesListener((store, getValueChange) => {
            const [hasChanged,_oldValue,newValue] = getValueChange(eventId);
            if (hasChanged) {
                console.log(`Event ${eventId} changed to ${newValue}`);
                listenerFn(newValue);
            }
        }, false);
    }
}