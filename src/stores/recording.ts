import { create } from "zustand";
import { ipc, on } from "@/lib/ipc";
import type { Status } from "@/lib/types";

const idle: Status = {
  state: "idle", recording: null, replay: null, error: null, game: null, profileId: "",
  target: { mode: "display", label: "", displayIndex: 1, width: 0, height: 0 },
};

interface RecordingState {
  status: Status;
  refresh: () => Promise<void>;
}

/** The engine is the single source of truth: the UI only mirrors what it reports. */
export const useRecording = create<RecordingState>((set) => ({
  status: idle,
  refresh: async () => set({ status: await ipc.getStatus() }),
}));

export function subscribeStatus() {
  void useRecording.getState().refresh();
  return on<Status>("engine-status", (status) => useRecording.setState({ status }));
}

/** Elapsed recording time in ms at `now`, excluding paused stretches. */
export function elapsedMs(status: Status, now: number): number {
  const r = status.recording;
  if (!r) return 0;
  return r.elapsedBaseMs + (r.runningSinceMs != null ? Math.max(0, now - r.runningSinceMs) : 0);
}

export const isRecording = (s: Status) => s.state === "recording" || s.state === "paused";
