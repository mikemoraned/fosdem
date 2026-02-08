/**
 * Creates a video player controller for a container
 * @param {string} playerId - The ID of the video element
 * @param {string} eventSelector - CSS selector for finding events with videos
 * @returns {Object|null} Controller object with updatePlaylist and loadVideo methods
 */
export function createVideoPlayer(playerId, eventSelector) {

    const container = document.getElementById(`${playerId}-container`);
    const video = document.getElementById(playerId);

    if (!container || !video) {
        console.warn(`Video player elements not found for ${playerId}`);
        return null;
    }

    const currentSpan = container.querySelector('.video-current');
    const totalSpan = container.querySelector('.video-total');
    const titleLink = container.querySelector('.video-title');

    let playlist = [];
    let currentIndex = 0;

    function updatePlaylist() {
        const events = document.querySelectorAll(eventSelector);
        playlist = [];
        events.forEach(event => {
            const videoLink = event.querySelector('a[data-type="video"]');
            if (videoLink) {
                const titleEl = event.querySelector('[data-event-title]');
                const title = titleEl ? titleEl.textContent : '';
                const eventUrl = event.dataset.eventUrl || '#';
                playlist.push({ url: videoLink.href, title, eventUrl });
            }
        });

        totalSpan.textContent = playlist.length;

        if (playlist.length > 0) {
            container.style.display = '';
            if (currentIndex >= playlist.length) {
                currentIndex = 0;
            }
            loadVideo(currentIndex);
        } else {
            container.style.display = 'none';
            titleLink.textContent = '';
            titleLink.href = '#';
        }

        return playlist.length;
    }

    function loadVideo(index) {
        if (index >= 0 && index < playlist.length) {
            currentIndex = index;
            video.src = playlist[index].url;
            currentSpan.textContent = index + 1;
            titleLink.textContent = playlist[index].title;
            titleLink.href = playlist[index].eventUrl;
        }
    }

    // Auto-advance on video end
    video.addEventListener('ended', () => {
        if (currentIndex < playlist.length - 1) {
            loadVideo(currentIndex + 1);
            video.play();
        }
    });

    // Playback controls (scoped to this container)
    const playBtns = container.querySelectorAll('.video-play');
    const pauseBtns = container.querySelectorAll('.video-pause');

    container.querySelectorAll('.video-prev').forEach(btn => {
        btn.addEventListener('click', () => {
            if (currentIndex > 0) {
                const wasPlaying = !video.paused;
                video.pause();
                loadVideo(currentIndex - 1);
                if (wasPlaying) video.play();
            }
        });
    });
    playBtns.forEach(btn => btn.addEventListener('click', () => video.play()));
    pauseBtns.forEach(btn => btn.addEventListener('click', () => video.pause()));
    container.querySelectorAll('.video-next').forEach(btn => {
        btn.addEventListener('click', () => {
            if (currentIndex < playlist.length - 1) {
                const wasPlaying = !video.paused;
                video.pause();
                loadVideo(currentIndex + 1);
                if (wasPlaying) video.play();
            }
        });
    });

    // Toggle play/pause button states based on video state
    video.addEventListener('play', () => {
        playBtns.forEach(btn => btn.disabled = true);
        pauseBtns.forEach(btn => btn.disabled = false);
    });
    video.addEventListener('pause', () => {
        playBtns.forEach(btn => btn.disabled = false);
        pauseBtns.forEach(btn => btn.disabled = true);
    });
    video.addEventListener('ended', () => {
        playBtns.forEach(btn => btn.disabled = false);
        pauseBtns.forEach(btn => btn.disabled = true);
    });

    // Observe DOM changes for dynamic updates
    const observer = new MutationObserver(updatePlaylist);
    document.querySelectorAll('[data-event-id][data-bookmark-status]').forEach(el => {
        observer.observe(el, { attributes: true, attributeFilter: ['data-bookmark-status'] });
    });

    // Initial playlist build (after a small delay to let any statuses settle)
    setTimeout(updatePlaylist, 100);

    return { updatePlaylist, loadVideo };
}
