/**
 * Connection state management hook.
 * Tracks connection status to the BlueBubbles server.
 */
import { useCallback } from "react";
import { useConnectionStore } from "@/store/connectionStore";
import { tauriConnect, tauriGetServerInfo, type ServerInfo } from "./useTauri";

export function useConnection() {
  const {
    status,
    serverInfo,
    error,
    setStatus,
    setServerInfo,
    setError,
  } = useConnectionStore();

  const connect = useCallback(
    async (address: string, password: string) => {
      setStatus("connecting");
      setError(null);

      try {
        const info = await tauriConnect(address, password);
        setServerInfo(info);
        setStatus("connected");
        return info;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        setStatus("disconnected");
        throw err;
      }
    },
    [setStatus, setServerInfo, setError]
  );

  const refreshServerInfo = useCallback(async () => {
    try {
      const info = await tauriGetServerInfo();
      setServerInfo(info);
      return info;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      return null;
    }
  }, [setServerInfo, setError]);

  return {
    status,
    serverInfo,
    error,
    connect,
    refreshServerInfo,
    isConnected: status === "connected",
    isConnecting: status === "connecting",
  };
}
