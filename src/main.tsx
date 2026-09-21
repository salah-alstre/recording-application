import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { boot } from "./boot";
import { App } from "./App";
import { subscribeDevices } from "@/stores/devices";
import { subscribeLibrary } from "@/stores/library";

void boot().then(() => {
  subscribeDevices();
  subscribeLibrary();
  createRoot(document.getElementById("root")!).render(<StrictMode><App /></StrictMode>);
});
