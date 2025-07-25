import { isDesktopApp } from "../api/core.ts";

if (isDesktopApp) {
  // Create a transparent drag region for desktop apps
  const dragRegion = document.createElement("div");
  dragRegion.className =
    "fixed top-0 left-0 right-0 h-8 z-[99] transparent select-none pointer-events-auto";
  dragRegion.setAttribute("data-tauri-drag-region", "");

  dragRegion.style.cssText = `
    pointer-events: auto;
    user-select: none;
    -webkit-app-region: drag;
  `;

  document.body.insertBefore(dragRegion, document.body.firstChild);
}
