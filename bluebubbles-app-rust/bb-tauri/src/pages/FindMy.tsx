/**
 * FindMy page - displays Find My devices and friends on an interactive map.
 * Uses react-leaflet with OpenStreetMap tiles to show device/friend markers.
 * Communicates with the BlueBubbles server via Tauri invoke commands.
 */
import { useState, useEffect, useCallback, useRef, useMemo, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { MapContainer, TileLayer, Marker, Popup, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { useConnectionStore } from "@/store/connectionStore";

// ─── Types ──────────────────────────────────────────────────────────────────

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

interface FindMyFriend {
  id: string;
  name: string;
  latitude: number | null;
  longitude: number | null;
  address: string | null;
  last_updated: number | null;
  status: string | null;
  locating_in_progress: boolean;
}

type ActiveTab = "devices" | "people";

// ─── Helpers ────────────────────────────────────────────────────────────────

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

function batteryColor(percent: number): string {
  if (percent <= 10) return "#FF3B30";
  if (percent <= 20) return "#FF9500";
  return "#34C759";
}

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].charAt(0).toUpperCase();
  return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase();
}

function statusColor(status: string | null): string {
  if (!status) return "var(--color-outline)";
  const s = status.toLowerCase();
  if (s === "live" || s === "sharing") return "#34C759";
  if (s === "shallow") return "#FF9500";
  return "var(--color-outline)";
}

function statusLabel(status: string | null): string {
  if (!status) return "Unknown";
  const s = status.toLowerCase();
  if (s === "live" || s === "sharing") return "Live";
  if (s === "shallow") return "Shallow";
  if (s === "legacy") return "Legacy";
  return status;
}

/** Detect if the current theme is a dark variant by checking the data-theme attribute. */
function isDarkTheme(): boolean {
  const theme = document.documentElement.getAttribute("data-theme") ?? "";
  return theme.includes("dark") || theme.includes("oled") || theme.includes("nord");
}

// ─── Custom Leaflet Marker Icons ────────────────────────────────────────────

function createSvgIcon(color: string, label: string): L.DivIcon {
  return L.divIcon({
    className: "",
    iconSize: [32, 40],
    iconAnchor: [16, 40],
    popupAnchor: [0, -42],
    html: `
      <svg width="32" height="40" viewBox="0 0 32 40" xmlns="http://www.w3.org/2000/svg">
        <path d="M16 0C7.164 0 0 7.164 0 16c0 12 16 24 16 24s16-12 16-24C32 7.164 24.836 0 16 0z" fill="${color}" />
        <circle cx="16" cy="15" r="10" fill="white" opacity="0.9" />
        <text x="16" y="19" text-anchor="middle" font-size="11" font-weight="600" fill="${color}" font-family="-apple-system,system-ui,sans-serif">${label}</text>
      </svg>
    `,
  });
}

// ─── Map Bounds Fitter ──────────────────────────────────────────────────────

function MapBoundsFitter({
  positions,
  focusPosition,
}: {
  positions: [number, number][];
  focusPosition: [number, number] | null;
}) {
  const map = useMap();

  useEffect(() => {
    if (focusPosition) {
      map.setView(focusPosition, 15, { animate: true });
      return;
    }
    if (positions.length === 0) return;
    if (positions.length === 1) {
      map.setView(positions[0], 14, { animate: true });
      return;
    }
    const bounds = L.latLngBounds(positions.map(([lat, lng]) => [lat, lng]));
    map.fitBounds(bounds, { padding: [40, 40], animate: true });
  }, [positions, focusPosition, map]);

  return null;
}

// ─── Main Component ─────────────────────────────────────────────────────────

// ─── Mock Data for Testing ──────────────────────────────────────────────────

const MOCK_DEVICES: FindMyDevice[] = [
  {
    id: "test-iphone-1",
    name: "Hassan's iPhone",
    model: "iPhone 14 Pro",
    device_class: "iPhone",
    raw_device_model: "iPhone15,2",
    battery_level: 0.85,
    battery_status: "200",
    latitude: 37.7749,
    longitude: -122.4194,
    location_timestamp: Date.now(),
    location_type: "GPS",
    address: "San Francisco, CA",
    is_old_location: false,
    is_online: true,
    is_mac: false,
    this_device: false,
    lost_mode_enabled: false,
  },
  {
    id: "test-macbook-1",
    name: "Hassan's MacBook Pro",
    model: "MacBook Pro 16\"",
    device_class: "MacBook",
    raw_device_model: "MacBookPro18,1",
    battery_level: 0.65,
    battery_status: "200",
    latitude: 40.7128,
    longitude: -74.0060,
    location_timestamp: Date.now() - 3600000,
    location_type: "WiFi",
    address: "New York, NY",
    is_old_location: false,
    is_online: true,
    is_mac: true,
    this_device: true,
    lost_mode_enabled: false,
  },
  {
    id: "test-airpods-1",
    name: "AirPods Pro",
    model: "AirPods Pro",
    device_class: "AirPods",
    raw_device_model: "AirPodsPro1,1",
    battery_level: 0.45,
    battery_status: "200",
    latitude: 34.0522,
    longitude: -118.2437,
    location_timestamp: Date.now() - 7200000,
    location_type: "Bluetooth",
    address: "Los Angeles, CA",
    is_old_location: false,
    is_online: false,
    is_mac: false,
    this_device: false,
    lost_mode_enabled: false,
  },
];

// Set to true to use mock data for testing
const USE_MOCK_DATA = false;

export function FindMy() {
  const navigate = useNavigate();
  const { status } = useConnectionStore();

  const [activeTab, setActiveTab] = useState<ActiveTab>("devices");
  const [devices, setDevices] = useState<FindMyDevice[]>([]);
  const [friends, setFriends] = useState<FindMyFriend[]>([]);
  const [loadingDevices, setLoadingDevices] = useState(true);
  const [loadingFriends, setLoadingFriends] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focusPosition, setFocusPosition] = useState<[number, number] | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const darkMode = useMemo(() => isDarkTheme(), []);

  // ── Data Fetching ───────────────────────────────────────────────────────

  const fetchDevices = useCallback(async () => {
    if (status !== "connected") {
      console.log("FindMy: Not connected, skipping device fetch");
      setLoadingDevices(false);
      return;
    }

    // Use mock data if enabled (for testing UI without server)
    if (USE_MOCK_DATA) {
      console.log("FindMy: Using mock data for testing");
      setTimeout(() => {
        setDevices(MOCK_DEVICES);
        setLoadingDevices(false);
      }, 500);
      return;
    }

    try {
      console.log("FindMy: Fetching devices from server...");
      const result = await invoke<FindMyDevice[]>("get_findmy_devices");
      console.log("FindMy devices response:", result);
      console.log("Number of devices:", result?.length);
      console.log("Devices state after set:", result);
      setDevices(result);
      setError(null);
    } catch (err) {
      console.error("FindMy: Error fetching devices:", err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoadingDevices(false);
    }
  }, [status]);

  const fetchFriends = useCallback(async () => {
    if (status !== "connected") {
      console.log("FindMy: Not connected, skipping friends fetch");
      setLoadingFriends(false);
      return;
    }
    try {
      console.log("FindMy: Fetching friends from server...");
      const result = await invoke<FindMyFriend[]>("get_findmy_friends");
      console.log("FindMy friends response:", result);
      console.log("Number of friends:", result?.length);
      setFriends(result);
    } catch (err) {
      // Friends might not be available on all servers, don't treat as fatal
      console.warn("FindMy: Failed to fetch friends:", err);
    } finally {
      setLoadingFriends(false);
    }
  }, [status]);

  const handleRefresh = useCallback(async () => {
    console.log("FindMy: Refreshing", activeTab);
    setRefreshing(true);
    setError(null);
    try {
      if (activeTab === "devices") {
        console.log("FindMy: Refreshing devices from server...");
        const result = await invoke<FindMyDevice[]>("refresh_findmy_devices");
        console.log("FindMy refresh devices response:", result);
        console.log("Number of devices after refresh:", result?.length);
        setDevices(result);
      } else {
        console.log("FindMy: Refreshing friends from server...");
        const result = await invoke<FindMyFriend[]>("refresh_findmy_friends");
        console.log("FindMy refresh friends response:", result);
        console.log("Number of friends after refresh:", result?.length);
        setFriends(result);
      }
    } catch (err) {
      console.error("FindMy: Error during refresh:", err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRefreshing(false);
    }
  }, [activeTab]);

  useEffect(() => {
    fetchDevices();
    fetchFriends();
  }, [fetchDevices, fetchFriends]);

  // Clear focus when switching tabs
  useEffect(() => {
    setFocusPosition(null);
    setSelectedId(null);
  }, [activeTab]);

  // ── Computed Values ─────────────────────────────────────────────────────

  const loading = activeTab === "devices" ? loadingDevices : loadingFriends;
  const items = activeTab === "devices" ? devices : friends;

  // Debug logging for render state
  useEffect(() => {
    console.log("FindMy Render State:", {
      activeTab,
      loading,
      itemsCount: items.length,
      devicesCount: devices.length,
      friendsCount: friends.length,
      status,
      error,
    });
  }, [activeTab, loading, items.length, devices.length, friends.length, status, error]);

  const allMapPositions = useMemo((): [number, number][] => {
    const positions: [number, number][] = [];
    if (activeTab === "devices") {
      devices.forEach((d) => {
        if (d.latitude != null && d.longitude != null) {
          positions.push([d.latitude, d.longitude]);
        }
      });
    } else {
      friends.forEach((f) => {
        if (f.latitude != null && f.longitude != null) {
          positions.push([f.latitude, f.longitude]);
        }
      });
    }
    return positions;
  }, [activeTab, devices, friends]);

  const deviceMarkerIcon = useMemo(
    () => createSvgIcon("#2C6BED", "\uD83D\uDCCD"),
    []
  );
  const friendMarkerIcon = useMemo(
    () => createSvgIcon("#34C759", "\uD83D\uDC64"),
    []
  );

  // ── Card Click Handler ──────────────────────────────────────────────────

  const handleCardClick = useCallback(
    (id: string, lat: number | null, lng: number | null) => {
      setSelectedId((prev) => (prev === id ? null : id));
      if (lat != null && lng != null) {
        setFocusPosition([lat, lng]);
      }
    },
    []
  );

  // ── Map Tile URL ────────────────────────────────────────────────────────

  const tileUrl = darkMode
    ? "https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
    : "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";

  const tileAttribution = darkMode
    ? '&copy; <a href="https://carto.com/">CARTO</a> &copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    : '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';

  // ── Styles ──────────────────────────────────────────────────────────────

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
    backgroundColor: "var(--color-background)",
  };

  const headerStyle: CSSProperties = {
    padding: "12px 24px",
    borderBottom: "1px solid var(--color-surface-variant)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexShrink: 0,
  };

  // ── Disconnected State ──────────────────────────────────────────────────

  if (status !== "connected") {
    return (
      <div style={containerStyle}>
        <div style={headerStyle}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <BackButton onClick={() => navigate("/")} />
            <h1 style={titleStyle}>Find My</h1>
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
          <span
            style={{
              fontSize: "var(--font-body-medium)",
              textAlign: "center",
              maxWidth: 340,
            }}
          >
            Connect to your BlueBubbles server to view Find My devices
          </span>
        </div>
      </div>
    );
  }

  // ── Main Render ─────────────────────────────────────────────────────────

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div style={headerStyle}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <BackButton onClick={() => navigate("/")} />
          <h1 style={titleStyle}>Find My</h1>
        </div>
        <button
          onClick={handleRefresh}
          disabled={refreshing}
          style={{
            padding: "6px 14px",
            borderRadius: 16,
            fontSize: "var(--font-label-large)",
            color: refreshing
              ? "var(--color-on-surface-variant)"
              : "var(--color-primary)",
            backgroundColor: "var(--color-primary-container)",
            cursor: refreshing ? "default" : "pointer",
            border: "none",
            display: "flex",
            alignItems: "center",
            gap: 6,
            opacity: refreshing ? 0.7 : 1,
            transition: "opacity 150ms ease",
          }}
          aria-label={refreshing ? "Refreshing locations" : "Refresh locations"}
        >
          {refreshing && <LoadingSpinner size={14} />}
          {refreshing ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      {/* Tab Bar */}
      <div
        style={{
          display: "flex",
          padding: "8px 24px 0",
          gap: 0,
          flexShrink: 0,
        }}
      >
        <TabButton
          label="Devices"
          count={devices.length}
          isActive={activeTab === "devices"}
          onClick={() => setActiveTab("devices")}
        />
        <TabButton
          label="People"
          count={friends.length}
          isActive={activeTab === "people"}
          onClick={() => setActiveTab("people")}
        />
      </div>

      {/* Error */}
      {error && (
        <div
          style={{
            margin: "8px 24px 0",
            padding: "10px 14px",
            borderRadius: 10,
            backgroundColor: "var(--color-error-container)",
            color: "var(--color-on-error-container)",
            fontSize: "var(--font-body-small)",
            display: "flex",
            flexDirection: "column",
            gap: 6,
          }}
        >
          <div style={{ fontWeight: 600 }}>Error loading FindMy data</div>
          <div>{error}</div>
          <div style={{ fontSize: "var(--font-label-small)", marginTop: 4 }}>
            Make sure FindMy is enabled on your BlueBubbles server and that you have granted permissions to access FindMy data.
          </div>
        </div>
      )}

      {/* Map */}
      <div
        style={{
          flex: "0 0 55%",
          minHeight: 200,
          position: "relative",
          borderBottom: "1px solid var(--color-surface-variant)",
        }}
      >
        <MapContainer
          center={[39.8283, -98.5795]}
          zoom={4}
          style={{ width: "100%", height: "100%" }}
          zoomControl={false}
          attributionControl={true}
        >
          <TileLayer url={tileUrl} attribution={tileAttribution} />
          <MapBoundsFitter
            positions={allMapPositions}
            focusPosition={focusPosition}
          />

          {/* Device markers */}
          {activeTab === "devices" &&
            devices
              .filter((d) => d.latitude != null && d.longitude != null)
              .map((d) => (
                <Marker
                  key={`device-${d.id || d.name}`}
                  position={[d.latitude!, d.longitude!]}
                  icon={deviceMarkerIcon}
                >
                  <Popup>
                    <div style={{ minWidth: 160 }}>
                      <div
                        style={{
                          fontWeight: 600,
                          fontSize: 14,
                          marginBottom: 4,
                        }}
                      >
                        {getDeviceIcon(d)} {d.name}
                      </div>
                      {d.address && (
                        <div
                          style={{
                            fontSize: 12,
                            color: "#666",
                            marginBottom: 2,
                          }}
                        >
                          {d.address}
                        </div>
                      )}
                      <div style={{ fontSize: 11, color: "#999" }}>
                        {d.model}
                        {d.battery_level != null &&
                          ` \u00B7 ${Math.round(d.battery_level * 100)}%`}
                      </div>
                    </div>
                  </Popup>
                </Marker>
              ))}

          {/* Friend markers */}
          {activeTab === "people" &&
            friends
              .filter((f) => f.latitude != null && f.longitude != null)
              .map((f) => (
                <Marker
                  key={`friend-${f.id || f.name}`}
                  position={[f.latitude!, f.longitude!]}
                  icon={friendMarkerIcon}
                >
                  <Popup>
                    <div style={{ minWidth: 160 }}>
                      <div
                        style={{
                          fontWeight: 600,
                          fontSize: 14,
                          marginBottom: 4,
                        }}
                      >
                        {f.name}
                      </div>
                      {f.address && (
                        <div
                          style={{
                            fontSize: 12,
                            color: "#666",
                            marginBottom: 2,
                          }}
                        >
                          {f.address}
                        </div>
                      )}
                      {f.last_updated && (
                        <div style={{ fontSize: 11, color: "#999" }}>
                          {formatLocationTime(f.last_updated)}
                        </div>
                      )}
                    </div>
                  </Popup>
                </Marker>
              ))}
        </MapContainer>

        {/* Map loading overlay */}
        {loading && (
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: "rgba(0,0,0,0.15)",
              zIndex: 1000,
            }}
          >
            <div
              style={{
                backgroundColor: "var(--color-surface)",
                borderRadius: 12,
                padding: "12px 20px",
                display: "flex",
                alignItems: "center",
                gap: 10,
                boxShadow: "var(--elevation-2)",
              }}
            >
              <LoadingSpinner size={18} />
              <span
                style={{
                  fontSize: "var(--font-body-medium)",
                  color: "var(--color-on-surface)",
                }}
              >
                Loading...
              </span>
            </div>
          </div>
        )}
      </div>

      {/* List Section */}
      <div
        ref={listRef}
        style={{
          flex: 1,
          overflow: "auto",
          padding: "12px 16px",
        }}
      >
        <AnimatePresence mode="wait">
          {!loading && items.length === 0 && (
            <motion.div
              key="empty"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                padding: 40,
                gap: 10,
                color: "var(--color-on-surface-variant)",
              }}
            >
              <span style={{ fontSize: 40 }}>
                {activeTab === "devices" ? "\uD83D\uDCCD" : "\uD83D\uDC65"}
              </span>
              <span
                style={{
                  fontSize: "var(--font-body-large)",
                  fontWeight: 500,
                }}
              >
                {activeTab === "devices"
                  ? "No Devices Found"
                  : "No People Found"}
              </span>
              <span
                style={{
                  fontSize: "var(--font-body-small)",
                  textAlign: "center",
                  maxWidth: 360,
                  lineHeight: 1.5,
                }}
              >
                {activeTab === "devices"
                  ? "Make sure Find My is enabled on your Mac and the BlueBubbles server has access to FindMy data."
                  : "Your Find My friends will appear here. Make sure location sharing is enabled in Find My on your Mac."}
              </span>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Device Cards */}
        {activeTab === "devices" && !loading && devices.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {devices.map((device) => (
              <DeviceCard
                key={device.id || device.name}
                device={device}
                isSelected={device.id === selectedId}
                onClick={() =>
                  handleCardClick(
                    device.id,
                    device.latitude,
                    device.longitude
                  )
                }
              />
            ))}
          </div>
        )}

        {/* Friend Cards */}
        {activeTab === "people" && !loading && friends.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {friends.map((friend) => (
              <FriendCard
                key={friend.id || friend.name}
                friend={friend}
                isSelected={friend.id === selectedId}
                onClick={() =>
                  handleCardClick(
                    friend.id,
                    friend.latitude,
                    friend.longitude
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

// ─── Shared Styles ──────────────────────────────────────────────────────────

const titleStyle: CSSProperties = {
  fontSize: "var(--font-title-large)",
  fontWeight: 700,
  color: "var(--color-on-surface)",
};

// ─── Sub-Components ─────────────────────────────────────────────────────────

function BackButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
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
        <path
          d="M9 1L1.5 9L9 17"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      Back
    </button>
  );
}

function TabButton({
  label,
  count,
  isActive,
  onClick,
}: {
  label: string;
  count: number;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        flex: 1,
        padding: "10px 0 8px",
        fontSize: "var(--font-body-medium)",
        fontWeight: isActive ? 600 : 400,
        color: isActive ? "var(--color-primary)" : "var(--color-on-surface-variant)",
        backgroundColor: "transparent",
        border: "none",
        borderBottom: isActive
          ? "2px solid var(--color-primary)"
          : "2px solid transparent",
        cursor: "pointer",
        transition: "all 150ms ease",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 6,
      }}
      role="tab"
      aria-selected={isActive}
    >
      {label}
      {count > 0 && (
        <span
          style={{
            fontSize: "var(--font-label-small)",
            backgroundColor: isActive
              ? "var(--color-primary)"
              : "var(--color-outline)",
            color: isActive
              ? "var(--color-on-primary)"
              : "var(--color-on-surface)",
            borderRadius: 10,
            padding: "1px 6px",
            fontWeight: 500,
            minWidth: 20,
            textAlign: "center",
          }}
        >
          {count}
        </span>
      )}
    </button>
  );
}

// ─── Device Card ────────────────────────────────────────────────────────────

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
        padding: "14px 16px",
        borderRadius: 14,
        border: isSelected
          ? "2px solid var(--color-primary)"
          : "1px solid var(--color-surface-variant)",
        backgroundColor: hovered
          ? "var(--color-surface-variant)"
          : "var(--color-surface)",
        cursor: "pointer",
        transition: "all 150ms ease",
        display: "flex",
        alignItems: "center",
        gap: 12,
      }}
    >
      {/* Icon */}
      <span style={{ fontSize: 26, lineHeight: 1, flexShrink: 0 }}>{icon}</span>

      {/* Info */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span
            style={{
              fontSize: "var(--font-body-medium)",
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
                fontSize: 9,
                color: "var(--color-primary)",
                backgroundColor: "var(--color-primary-container)",
                padding: "1px 5px",
                borderRadius: 5,
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
                fontSize: 9,
                color: "var(--color-error)",
                backgroundColor: "var(--color-error-container)",
                padding: "1px 5px",
                borderRadius: 5,
                fontWeight: 500,
                flexShrink: 0,
              }}
            >
              Lost
            </span>
          )}
        </div>

        {/* Address or model */}
        <span
          style={{
            fontSize: "var(--font-body-small)",
            color: "var(--color-on-surface-variant)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            display: "block",
          }}
        >
          {hasLocation && device.address ? device.address : device.model}
        </span>

        {/* Time + coordinates */}
        {hasLocation && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              marginTop: 2,
            }}
          >
            {locationTimeStr && (
              <span
                style={{
                  fontSize: "var(--font-label-small)",
                  color: device.is_old_location
                    ? "var(--color-error)"
                    : "var(--color-outline)",
                }}
              >
                {locationTimeStr}
              </span>
            )}
          </div>
        )}
        {!hasLocation && (
          <span
            style={{
              fontSize: "var(--font-label-small)",
              color: "var(--color-outline)",
              fontStyle: "italic",
              marginTop: 2,
              display: "block",
            }}
          >
            No location
          </span>
        )}
      </div>

      {/* Right side: battery + status */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "flex-end",
          gap: 6,
          flexShrink: 0,
        }}
      >
        {/* Online status dot */}
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            backgroundColor: device.is_online ? "#34C759" : "var(--color-outline)",
          }}
          title={device.is_online ? "Online" : "Offline"}
        />

        {/* Battery */}
        {batteryPercent != null && (
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <div
              style={{
                width: 40,
                height: 5,
                borderRadius: 3,
                backgroundColor: "var(--color-surface-variant)",
                overflow: "hidden",
              }}
            >
              <div
                style={{
                  width: `${Math.max(batteryPercent, 3)}%`,
                  height: "100%",
                  borderRadius: 3,
                  backgroundColor: batteryColor(batteryPercent),
                  transition: "width 300ms ease",
                }}
              />
            </div>
            <span
              style={{
                fontSize: 10,
                color:
                  batteryPercent > 20
                    ? "var(--color-on-surface-variant)"
                    : "var(--color-error)",
                fontWeight: 500,
                minWidth: 28,
                textAlign: "right",
              }}
            >
              {batteryPercent}%
            </span>
          </div>
        )}
      </div>
    </motion.div>
  );
}

// ─── Friend Card ────────────────────────────────────────────────────────────

interface FriendCardProps {
  friend: FindMyFriend;
  isSelected: boolean;
  onClick: () => void;
}

function FriendCard({ friend, isSelected, onClick }: FriendCardProps) {
  const [hovered, setHovered] = useState(false);

  const hasLocation = friend.latitude != null && friend.longitude != null;
  const initials = getInitials(friend.name);
  const sColor = statusColor(friend.status);
  const sLabel = statusLabel(friend.status);

  return (
    <motion.div
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      whileTap={{ scale: 0.98 }}
      style={{
        padding: "14px 16px",
        borderRadius: 14,
        border: isSelected
          ? "2px solid var(--color-primary)"
          : "1px solid var(--color-surface-variant)",
        backgroundColor: hovered
          ? "var(--color-surface-variant)"
          : "var(--color-surface)",
        cursor: "pointer",
        transition: "all 150ms ease",
        display: "flex",
        alignItems: "center",
        gap: 12,
      }}
    >
      {/* Avatar */}
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: "50%",
          background: "linear-gradient(135deg, #34C759, #30B550)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
          position: "relative",
        }}
      >
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "#fff",
            letterSpacing: 0.5,
          }}
        >
          {initials}
        </span>
        {/* Status dot on avatar */}
        <span
          style={{
            position: "absolute",
            bottom: -1,
            right: -1,
            width: 10,
            height: 10,
            borderRadius: "50%",
            backgroundColor: sColor,
            border: "2px solid var(--color-surface)",
          }}
        />
      </div>

      {/* Info */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <span
          style={{
            fontSize: "var(--font-body-medium)",
            fontWeight: 600,
            color: "var(--color-on-surface)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            display: "block",
          }}
        >
          {friend.name}
        </span>

        <span
          style={{
            fontSize: "var(--font-body-small)",
            color: "var(--color-on-surface-variant)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            display: "block",
          }}
        >
          {hasLocation && friend.address
            ? friend.address
            : hasLocation
            ? `${friend.latitude!.toFixed(4)}, ${friend.longitude!.toFixed(4)}`
            : "No location available"}
        </span>

        {friend.last_updated && (
          <span
            style={{
              fontSize: "var(--font-label-small)",
              color: "var(--color-outline)",
              marginTop: 2,
              display: "block",
            }}
          >
            {formatLocationTime(friend.last_updated)}
          </span>
        )}
      </div>

      {/* Right: status badge */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "flex-end",
          gap: 4,
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: 10,
            fontWeight: 500,
            color: sColor,
            backgroundColor:
              sColor === "#34C759"
                ? "rgba(52,199,89,0.12)"
                : sColor === "#FF9500"
                ? "rgba(255,149,0,0.12)"
                : "var(--color-surface-variant)",
            padding: "2px 8px",
            borderRadius: 8,
          }}
        >
          {sLabel}
        </span>
        {friend.locating_in_progress && (
          <span
            style={{
              fontSize: 9,
              color: "var(--color-on-surface-variant)",
              fontStyle: "italic",
            }}
          >
            Locating...
          </span>
        )}
      </div>
    </motion.div>
  );
}
