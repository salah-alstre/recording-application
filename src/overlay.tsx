import { createRoot } from "react-dom/client";
import { boot } from "./boot";
import { Overlay } from "@/features/overlay/Overlay";
import { subscribeDevices } from "@/stores/devices";
import { subscribeLibrary } from "@/stores/library";

void boot({ transparent: true }).then(() => {
  subscribeDevices();
  subscribeLibrary();
  createRoot(document.getElementById("root")!).render(<Overlay />);
});
