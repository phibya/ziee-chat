import { isDesktopApp } from "../api/core.ts";

if (isDesktopApp) {
  // Create a transparent drag region for desktop apps
  const dragRegion = document.createElement("div");
  dragRegion.className =
    "fixed top-0 left-0 right-0 h-8 z-[99] transparent select-none pointer-events-auto";
  dragRegion.setAttribute("data-tauri-drag-region", "");
  document.body.insertBefore(dragRegion, document.body.firstChild);

  // setTimeout(() => {
  //   //open blank window and render random text
  //   const appWindow = new Window("uniqueLabel");
  //
  //   console.log({ appWindow });
  //
  //   appWindow.once("tauri://created", async function () {
  //     // `new Webview` Should be called after the window is successfully created,
  //     // or webview may not be attached to the window since window is not created yet.
  //
  //     console.log("Window created successfully");
  //
  //     const webview = new Webview(appWindow, "theUniqueLabel", {
  //       url: "https://github.com/tauri-apps/tauri",
  //
  //       // create a webview with specific logical position and size
  //       x: 0,
  //       y: 0,
  //       width: 800,
  //       height: 600,
  //     });
  //
  //     webview.once("tauri://created", function () {
  //       // webview successfully created
  //     });
  //     webview.once("tauri://error", function (e) {
  //       // an error happened creating the webview
  //     });
  //
  //     // emit an event to the backend
  //     await webview.emit("some-event", "data");
  //     // listen to an event from the backend
  //     const unlisten = await webview.listen("event-name", (e) => {});
  //     unlisten();
  //   });
  // }, 3000);
}
