import { create } from "zustand";
import { ipc, on } from "@/lib/ipc";
import type { Clip, ClipQuery } from "@/lib/types";

interface LibraryState {
  clips: Clip[];
  loading: boolean;
  loaded: boolean;
  /** True while the backend is (re)generating thumbnails. */
  thumbsBusy: boolean;
  query: ClipQuery;
  setQuery: (q: Partial<ClipQuery>) => void;
  refresh: () => Promise<void>;
  scan: () => Promise<void>;
}

export const useLibrary = create<LibraryState>((set, get) => ({
  clips: [],
  loading: false,
  loaded: false,
  thumbsBusy: false,
  query: { section: "all", search: "", sort: "newest" },
  setQuery: (q) => {
    set({ query: { ...get().query, ...q } });
    void get().refresh();
  },
  refresh: async () => {
    set({ loading: true });
    try {
      set({ clips: await ipc.listClips(get().query), loaded: true });
    } finally {
      set({ loading: false });
    }
  },
  scan: async () => {
    await ipc.scanLibrary();
    await get().refresh();
  },
}));

export function subscribeLibrary() {
  void useLibrary.getState().refresh();
  void ipc.thumbsBusy().then((thumbsBusy) => useLibrary.setState({ thumbsBusy })).catch(() => undefined);
  const subs = [
    on("library-changed", () => void useLibrary.getState().refresh()),
    on<boolean>("thumbs-busy", (thumbsBusy) => useLibrary.setState({ thumbsBusy })),
  ];
  return Promise.all(subs).then((us) => () => us.forEach((u) => u()));
}
