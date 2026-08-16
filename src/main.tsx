import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
}

const root = document.getElementById("root");
if (!root) throw new Error("fatal: #root element not found");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>
);
