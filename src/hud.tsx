import { createRoot } from "react-dom/client";
import { boot } from "./boot";
import { HudPill, StatsHud, ToastStack } from "@/features/hud/Hud";

const kind = new URLSearchParams(location.search).get("kind");

void boot({ transparent: true }).then(() => {
  const View = kind === "stats" ? StatsHud : kind === "toast" ? ToastStack : HudPill;
  createRoot(document.getElementById("root")!).render(<View />);
});
