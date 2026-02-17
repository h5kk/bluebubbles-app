/**
 * FindMy page - displays Find My device locations.
 * Communicates with the BlueBubbles server to retrieve device data.
 */
import { useState, useEffect, useCallback, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { useConnectionStore } from "@/store/connectionStore";

interface FindMyDevice {
  id: string;
  name: string;
  model: string;
  device_class: string | null;
  raw_device_model: string | null;
  battery_level: number | null;
  battery_status: string | null;
  latitude: number | null;
  longitude: number | null;
  location_timestamp: number | null;
  location_type: string | null;
  address: string | null;
  is_old_location: boolean;
  is_online: boolean;
  is_mac: boolean;
  this_device: boolean;
  lost_mode_enabled: boolean;
}

/** Format an epoch-millisecond timestamp into a human-readable relative string. */
function formatLocationTime(epochMs: number): string {
  const now = Date.now();
  const diffMs = now - epochMs;
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return "Just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDays = Math.floor(diffHr / 24);
  if (diffDays < 7) return `${diffDays}d ago`;

  const date = new Date(epochMs);
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

/** Get a device icon based on device class / model info. */
function getDeviceIcon(device: FindMyDevice): string {
  const cls = (device.device_class ?? "").toLowerCase();
  const model = (device.raw_device_model ?? device.model ?? "").toLowerCase();

  if (cls.includes("macbook") || model.includes("macbook")) return "\uD83D\uDCBB";
  if (cls.includes("imac") || model.includes("imac")) return "\uD83D\uDDA5\uFE0F";
  if (cls.includes("mac") || device.is_mac) return "\uD83D\uDDA5\uFE0F";
  if (cls.includes("iphone") || model.includes("iphone")) return "\uD83D\uDCF1";
  if (cls.includes("ipad") || model.includes("ipad")) return "\uD83D\uDCF1";
  if (cls.includes("watch") || model.includes("watch")) return "\u231A";
  if (cls.includes("airpods") || model.includes("airpods")) return "\uD83C\uDFA7";
  if (cls === "b389" || model.includes("airtag")) return "\uD83D\uDD34";
  return "\uD83D\uDCCD";
}

/** Battery bar color based on level. */
function batteryColor(percent: number): string {
  if (percent <= 10) return "#FF3B30";
  if (percent <= 20) return "#FF9500";
  return "#34C759";
}

export function FindMy() {
  const navigate = useNavigate();
  const { status } = useConnectionStore();
  const [devices, setDevices] = useState<FindMyDevice[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);

  const fetchDevices = useCallback(async () => {
    if (status !== "connected") {
      setLoading(false);
      return;
    }

    setError(null);
    try {
      const result = await invoke<FindMyDevice[]>("get_findmy_devices");
      setDevices(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [status]);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchDevices();
  }, [fetchDevices]);

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

  const backButton = (
    <button
      onClick={() => navigate("/")}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 4,
        padding: "4px 8px 4px 2px",
        borderRadius: 8,
        color: "var(--color-primary)",
        cursor: "pointer",
        backgroundColor: "transparent",
        fontSize: "var(--font-body-medium)",
        fontWeight: 400,
        border: "none",
      }}
      aria-label="Back to chats"
    >
      <svg width="10" height="18" viewBox="0 0 10 18" fill="none">
        <path d="M9 1L1.5 9L9 17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
      Back
    </button>
  );

  if (status !== "connected") {
    return (
      <div style={containerStyle}>
        <div style={headerStyle}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            {backButton}
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
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          {backButton}
          <h1
            style={{
              fontSize: "var(--font-title-large)",
              fontWeight: 700,
              color: "var(--color-on-surface)",
            }}
          >
            Find My
          </h1>
          {!loading && devices.length > 0 && (
            <span
              style={{
                fontSize: "var(--font-label-medium)",
                color: "var(--color-on-surface-variant)",
                fontWeight: 400,
              }}
            >
              {devices.length} {devices.length === 1 ? "device" : "devices"}
            </span>
          )}
        </div>
        <button
          onClick={handleRefresh}
          disabled={refreshing}
          style={{
            padding: "6px 14px",
            borderRadius: 16,
            fontSize: "var(--font-label-large)",
            color: refreshing ? "var(--color-on-surface-variant)" : "var(--color-primary)",
            backgroundColor: "var(--color-primary-container)",
            cursor: refreshing ? "default" : "pointer",
            border: "none",
            display: "flex",
            alignItems: "center",
            gap: 6,
            opacity: refreshing ? 0.7 : 1,
            transition: "opacity 150ms ease",
          }}
        >
          {refreshing && <LoadingSpinner size={14} />}
          {refreshing ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      <div style={{ flex: 1, overflow: "auto", padding: 24 }}>
        {loading && (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
              padding: 48,
              flexDirection: "column",
              gap: 12,
            }}
          >
            <LoadingSpinner size={28} />
            <span style={{ fontSize: "var(--font-body-medium)", color: "var(--color-on-surface-variant)" }}>
              Loading devices...
            </span>
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
            <span style={{ fontSize: "var(--font-body-medium)", textAlign: "center", maxWidth: 400 }}>
              Make sure Find My is enabled on your Mac and the BlueBubbles server
              has access to FindMy data. Note: macOS Sequoia and later may not support
              device location reading.
            </span>
          </div>
        )}

        {/* Device cards */}
        {!loading && devices.length > 0 && (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(300px, 1fr))",
              gap: 16,
            }}
          >
            {devices.map((device) => (
              <DeviceCard
                key={device.id || device.name}
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
        )}
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

  const icon = getDeviceIcon(device);
  const hasLocation = device.latitude != null && device.longitude != null;
  const locationTimeStr = device.location_timestamp
    ? formatLocationTime(device.location_timestamp)
    : null;

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
        gap: 12,
      }}
    >
      {/* Header: icon + name + status dot */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
        }}
      >
        <span style={{ fontSize: 28, lineHeight: 1 }}>{icon}</span>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span
              style={{
                fontSize: "var(--font-body-large)",
                fontWeight: 600,
                color: "var(--color-on-surface)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {device.name}
            </span>
            {device.this_device && (
              <span
                style={{
                  fontSize: "var(--font-label-small)",
                  color: "var(--color-primary)",
                  backgroundColor: "var(--color-primary-container)",
                  padding: "1px 6px",
                  borderRadius: 6,
                  fontWeight: 500,
                  flexShrink: 0,
                }}
              >
                This Device
              </span>
            )}
            {device.lost_mode_enabled && (
              <span
                style={{
                  fontSize: "var(--font-label-small)",
                  color: "var(--color-error)",
                  backgroundColor: "var(--color-error-container)",
                  padding: "1px 6px",
                  borderRadius: 6,
                  fontWeight: 500,
                  flexShrink: 0,
                }}
              >
                Lost
              </span>
            )}
          </div>
          <span
            style={{
              fontSize: "var(--font-body-small)",
              color: "var(--color-on-surface-variant)",
            }}
          >
            {device.model}
          </span>
        </div>
        <span
          style={{
            width: 10,
            height: 10,
            borderRadius: "50%",
            backgroundColor: device.is_online
              ? "#34C759"
              : "var(--color-outline)",
            flexShrink: 0,
          }}
          title={device.is_online ? "Online" : "Offline"}
        />
      </div>

      {/* Battery */}
      {batteryPercent != null && (
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div
            style={{
              flex: 1,
              height: 6,
              borderRadius: 3,
              backgroundColor: "var(--color-surface-variant)",
              overflow: "hidden",
              maxWidth: 120,
            }}
          >
            <div
              style={{
                width: `${Math.max(batteryPercent, 2)}%`,
                height: "100%",
                borderRadius: 3,
                backgroundColor: batteryColor(batteryPercent),
                transition: "width 300ms ease",
              }}
            />
          </div>
          <span
            style={{
              fontSize: "var(--font-label-small)",
              color:
                batteryPercent > 20
                  ? "var(--color-on-surface-variant)"
                  : "var(--color-error)",
              fontWeight: 500,
              minWidth: 32,
            }}
          >
            {batteryPercent}%
          </span>
        </div>
      )}

      {/* Location info */}
      {hasLocation && (
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          {device.address && (
            <span
              style={{
                fontSize: "var(--font-body-small)",
                color: "var(--color-on-surface-variant)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
              title={device.address}
            >
              {device.address}
            </span>
          )}
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span
              style={{
                fontSize: "var(--font-label-small)",
                color: "var(--color-outline)",
              }}
            >
              {device.latitude!.toFixed(4)}, {device.longitude!.toFixed(4)}
            </span>
            {locationTimeStr && (
              <>
                <span style={{ fontSize: "var(--font-label-small)", color: "var(--color-outline)" }}>
                  {"\u00B7"}
                </span>
                <span
                  style={{
                    fontSize: "var(--font-label-small)",
                    color: device.is_old_location ? "var(--color-error)" : "var(--color-outline)",
                  }}
                >
                  {locationTimeStr}
                </span>
              </>
            )}
          </div>
        </div>
      )}

      {!hasLocation && (
        <span
          style={{
            fontSize: "var(--font-body-small)",
            color: "var(--color-outline)",
            fontStyle: "italic",
          }}
        >
          No location available
        </span>
      )}
    </motion.div>
  );
}
