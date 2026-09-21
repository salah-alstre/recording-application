import { ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import type { AppError } from "@/lib/types";

/** Engine actions surface their failures through `status.error` and toasts, so the promise is not re-thrown. */
const quiet = (p: Promise<unknown>) => p.catch(() => undefined);

export const actions = {
  toggleRecord: () => quiet(ipc.toggleRecording()),
  pause: () => quiet(ipc.pauseRecording()),
  resume: () => quiet(ipc.resumeRecording()),
  toggleReplay: () => quiet(ipc.toggleReplay()),
  saveReplay: () => quiet(ipc.saveReplay()),
  screenshot: (mode?: string) => quiet(ipc.takeScreenshot(mode)),
  marker: () => quiet(ipc.addMarker()),
  toggleMic: () => quiet(ipc.toggleMic()),
  toggleCamera: () => quiet(ipc.toggleCamera()),
};

/** Runs any other backend call and shows its failure as a banner. */
export async function attempt<T>(p: Promise<T>): Promise<T | undefined> {
  try {
    return await p;
  } catch (e) {
    useUi.getState().setError(e as AppError);
    return undefined;
  }
}
