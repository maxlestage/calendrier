import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initTheme } from "./theme";
import "./styles.css";

// Before the first paint, so the app never flashes the wrong theme.
initTheme();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

/** Fade the launch screen out once the app has actually painted a frame. */
function dismissSplash(): void {
  const splash = document.getElementById("splash");
  if (!splash) return;
  splash.classList.add("done");
  splash.addEventListener("transitionend", () => splash.remove(), { once: true });
  // Belt and braces: remove it even if the transition never fires.
  window.setTimeout(() => splash.remove(), 1000);
}

// Two frames: the first commits React's output, the second paints it.
requestAnimationFrame(() => requestAnimationFrame(dismissSplash));
