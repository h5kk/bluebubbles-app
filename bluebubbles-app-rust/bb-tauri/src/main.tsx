/**
 * React entry point.
 * Mounts the App component and imports global styles.
 */
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/globals.css";
import "./styles/themes.css";
import "./styles/animations.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Root element not found. Make sure index.html has a div#root.");
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
