
const TABLET_MIN_WIDTH = 481;

export function init() {
    const detailElements = document.querySelectorAll("details.event");
    if (window.innerWidth >= TABLET_MIN_WIDTH) {
        detailElements.forEach((el) => {
            el.open = true;
        });
    }
}