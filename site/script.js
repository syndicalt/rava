for (const button of document.querySelectorAll("[data-copy-command]")) {
  button.setAttribute("aria-label", `Copy command: ${button.dataset.copyCommand}`);

  button.addEventListener("click", async () => {
    const original = button.textContent;
    await navigator.clipboard.writeText(button.dataset.copyCommand);
    button.textContent = "Copied";
    window.setTimeout(() => {
      button.textContent = original;
    }, 1400);
  });
}
