import { create } from "zustand";
import { ipc, on } from "@/lib/ipc";
import type { Devices, Hardware, MonitorInfo, Profile, WindowInfo } from "@/lib/types";

interface DeviceState {
  hardware: Hardware | null;
  monitors: MonitorInfo[];
  audio: Devices;
  cameras: string[];
  windows: WindowInfo[];
  profiles: Profile[];
  loadHardware: () => Promise<void>;
  loadMonitors: () => Promise<void>;
  loadAudio: () => Promise<void>;
  loadCameras: () => Promise<void>;
  loadWindows: () => Promise<void>;
  loadProfiles: () => Promise<void>;
}

export const useDevices = create<DeviceState>((set) => ({
  hardware: null,
  monitors: [],
  audio: { inputs: [], outputs: [] },
  cameras: [],
  windows: [],
  profiles: [],
  loadHardware: async () => set({ hardware: await ipc.getHardware() }),
  loadMonitors: async () => set({ monitors: await ipc.listMonitors() }),
  loadAudio: async () => set({ audio: await ipc.listAudioDevices() }),
  loadCameras: async () => set({ cameras: await ipc.listCameras() }),
  loadWindows: async () => set({ windows: await ipc.listWindows() }),
  loadProfiles: async () => set({ profiles: await ipc.listProfiles() }),
}));

export function subscribeDevices() {
  const s = useDevices.getState();
  void s.loadHardware();
  void s.loadMonitors();
  void s.loadProfiles();
  const subs = [
    on<Hardware>("hardware-ready", (hardware) => useDevices.setState({ hardware })),
    on("audio-devices-changed", () => void useDevices.getState().loadAudio()),
  ];
  return () => subs.forEach((p) => void p.then((u) => u()));
}
