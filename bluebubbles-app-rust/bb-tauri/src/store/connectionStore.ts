/**
 * Connection store - manages server connection state.
 */
import { create } from "zustand";
import type { ServerInfo } from "@/hooks/useTauri";

export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error";

interface ConnectionState {
  status: ConnectionStatus;
  serverInfo: ServerInfo | null;
  error: string | null;

  setStatus: (status: ConnectionStatus) => void;
  setServerInfo: (info: ServerInfo) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

export const useConnectionStore = create<ConnectionState>((set) => ({
  status: "disconnected",
  serverInfo: null,
  error: null,

  setStatus: (status) => set({ status }),
  setServerInfo: (serverInfo) => set({ serverInfo }),
  setError: (error) => set({ error }),
  reset: () =>
    set({
      status: "disconnected",
      serverInfo: null,
      error: null,
    }),
}));
