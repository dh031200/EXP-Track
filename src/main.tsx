import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { History } from "./pages/History";

// Determine which component to render based on the current URL
const pathname = window.location.pathname;
const Component = pathname === "/history" ? History : App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Component />
  </React.StrictMode>,
);
