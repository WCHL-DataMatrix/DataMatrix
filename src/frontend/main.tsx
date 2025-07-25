import React from "react";
import ReactDOM from "react-dom/client";
import App from "./src/App"; // 경로 확인
import "./src/index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
