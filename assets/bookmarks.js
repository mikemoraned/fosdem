document.addEventListener("DOMContentLoaded", () => {
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
});
  