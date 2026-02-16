/**
 * FindMy page - displays Find My device locations.
 * Communicates with the BlueBubbles server to retrieve device data.
 */
import { useState, useEffect, useCallback, type CSSProperties } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { useConnectionStore } from "@/store/connectionStore";

interface FindMyDevice {
  id: string;
  name: string;
  model: string;
  battery_level: number | null;
  battery_status: string | null;
  latitude: number | null;
  longitude: number | null;
  location_timestamp: string | null;
  is_online: boolean;
}

export function FindMy() {
  const { status } = useConnectionStore();
  const [devices, setDevices] = useState<FindMyDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);

  const fetchDevices = useCallback(async () => {
    if (status !== "connected") {
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      // This command would need to be added to the Tauri backend
      // For now we call invoke and handle gracefully if not available
      const result = await invoke<FindMyDevice[]>("get_findmy_devices").catch(
        () => [] as FindMyDevice[]
      );
      setDevices(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [status]);

  useEffect(() => {
    fetchDevices();
  }, [fetchDevices]);

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    padding: "16px 24px",
    borderBottom: "1px solid var(--color-surface-variant)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexShrink: 0,
  };

  if (status !== "connected") {
    return (
      <div style={containerStyle}>
        <div style={headerStyle}>
          <h1
            style={{
              fontSize: "var(--font-title-large)",
              fontWeight: 700,
              color: "var(--color-on-surface)",
            }}
          >
            Find My
          </h1>
        </div>
        <div
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: 12,
            color: "var(--color-on-surface-variant)",
          }}
        >
          <span style={{ fontSize: 48 }}>{"\uD83D\uDCCD"}</span>
          <span style={{ fontSize: "var(--font-body-large)", fontWeight: 500 }}>
            Not Connected
          </span>
          <span style={{ fontSize: "var(--font-body-medium)", textAlign: "center" }}>
            Connect to your BlueBubbles server to view Find My devices
          </span>
        </div>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      <div style={headerStyle}>
        <h1
          style={{
            fontSize: "var(--font-title-large)",
            fontWeight: 700,
            color: "var(--color-on-surface)",
          }}
        >
          Find My
        </h1>
        <button
          onClick={fetchDevices}
          style={{
            padding: "6px 14px",
            borderRadius: 16,
            fontSize: "var(--font-label-large)",
            color: "var(--color-primary)",
            backgroundColor: "var(--color-primary-container)",
            cursor: "pointer",
          }}
        >
          Refresh
        </button>
      </div>

      <div style={{ flex: 1, overflow: "auto", padding: 24 }}>
        {loading && (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              padding: 32,
            }}
          >
            <LoadingSpinner size={28} />
          </div>
        )}

        {error && (
          <div
            style={{
              padding: "12px 16px",
              borderRadius: 12,
              backgroundColor: "var(--color-error-container)",
              color: "var(--color-error)",
              fontSize: "var(--font-body-medium)",
              marginBottom: 16,
            }}
          >
            {error}
          </div>
        )}

        {!loading && devices.length === 0 && !error && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              padding: 48,
              gap: 12,
              color: "var(--color-on-surface-variant)",
            }}
          >
            <span style={{ fontSize: 48 }}>{"\uD83D\uDCCD"}</span>
            <span style={{ fontSize: "var(--font-body-large)", fontWeight: 500 }}>
              No Devices Found
            </span>
            <span style={{ fontSize: "var(--font-body-medium)", textAlign: "center" }}>
              Make sure Find My is enabled on your Mac and the Private API is active
            </span>
          </div>
        )}

        {/* Device cards */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
            gap: 16,
          }}
        >
          {devices.map((device) => (
            <DeviceCard
              key={device.id}
              device={device}
              isSelected={device.id === selectedDevice}
              onClick={() =>
                setSelectedDevice(
                  device.id === selectedDevice ? null : device.id
                )
              }
            />
          ))}
        </div>
      </div>
    </div>
  );
}

interface DeviceCardProps {
  device: FindMyDevice;
  isSelected: boolean;
  onClick: () => void;
}

function DeviceCard({ device, isSelected, onClick }: DeviceCardProps) {
  const [hovered, setHovered] = useState(false);

  const batteryPercent =
    device.battery_level != null ? Math.round(device.battery_level * 100) : null;

  return (
    <motion.div
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      whileTap={{ scale: 0.98 }}
      style={{
        padding: 20,
        borderRadius: 16,
        border: isSelected
          ? "2px solid var(--color-primary)"
          : "1px solid var(--color-surface-variant)",
        backgroundColor: hovered
          ? "var(--color-surface-variant)"
          : "var(--color-surface)",
        cursor: "pointer",
        transition: "all 150ms ease",
        display: "flex",
        flexDirection: "column",
        gap: 10,
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span
          style={{
            fontSize: "var(--font-body-large)",
            fontWeight: 600,
            color: "var(--color-on-surface)",
          }}
        >
          {device.name}
        </span>
        <span
          style={{
            width: 10,
            height: 10,
            borderRadius: "50%",
            backgroundColor: device.is_online
              ? "#43CC47"
              : "var(--color-outline)",
            flexShrink: 0,
          }}
        />
      </div>

      <span
        style={{
          fontSize: "var(--font-body-medium)",
          color: "var(--color-on-surface-variant)",
        }}
      >
        {device.model}
      </span>

      {batteryPercent != null && (
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{ fontSize: 14 }}>
            {batteryPercent > 20 ? "\uD83D\uDD0B" : "\uD83E\uDEAB"}
          </span>
          <span
            style={{
              fontSize: "var(--font-body-small)",
              color:
                batteryPercent > 20
                  ? "var(--color-on-surface-variant)"
                  : "var(--color-error)",
            }}
          >
            {batteryPercent}%
          </span>
        </div>
      )}

      {device.latitude != null && device.longitude != null && (
        <span
          style={{
            fontSize: "var(--font-label-small)",
            color: "var(--color-outline)",
          }}
        >
          {device.latitude.toFixed(4)}, {device.longitude.toFixed(4)}
        </span>
      )}
    </motion.div>
  );
}
