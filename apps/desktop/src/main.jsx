import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App.jsx";
import "./styles.css";

const injectedBackend = globalThis.__FUTUREAERO_BACKEND__;

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <App backend={injectedBackend} />
  </React.StrictMode>
);
