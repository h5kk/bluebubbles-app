/**
 * LocationMapCard - Collapsible map card showing contact's Find My location.
 * Displays in the Info tab of ContactDetailsSidebar.
 */
import { useState, useMemo, type CSSProperties } from "react";
import { MapContainer, TileLayer, Marker } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import type { FindMyFriend } from "@/store/findMyStore";
import { Avatar } from "./Avatar";

interface LocationMapCardProps {
  friend: FindMyFriend | null;
  contactName: string;
  contactAddress: string;
  onExpand: () => void;
}

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
  return `${diffDays}d ago`;
}

function isDarkTheme(): boolean {
  const theme = document.documentElement.getAttribute("data-theme") ?? "";
  return theme.includes("dark") || theme.includes("oled") || theme.includes("nord");
}

function createAvatarMarkerIcon(name: string, address: string): L.DivIcon {
  return L.divIcon({
    className: "",
    iconSize: [40, 40],
    iconAnchor: [20, 20],
    html: `
      <div style="
        width: 40px;
        height: 40px;
        border-radius: 50%;
        overflow: hidden;
        border: 3px solid #fff;
        box-shadow: 0 2px 8px rgba(0,0,0,0.3);
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-weight: 600;
        font-size: 16px;
      ">
        ${name.charAt(0).toUpperCase()}
      </div>
    `,
  });
}

export function LocationMapCard({
  friend,
  contactName,
  contactAddress,
  onExpand,
}: LocationMapCardProps) {
  const [mapReady, setMapReady] = useState(false);

  const hasLocation = friend && friend.latitude != null && friend.longitude != null;

  const isLive = useMemo(() => {
    if (!friend || !friend.status) return false;
    const status = friend.status.toLowerCase();
    return status === "live" || status === "sharing";
  }, [friend]);

  const locationText = useMemo(() => {
    if (!friend) return "Location sharing not available";
    if (!hasLocation) return "Location not available";
    if (friend.address) return friend.address;
    return `${friend.latitude?.toFixed(4)}, ${friend.longitude?.toFixed(4)}`;
  }, [friend, hasLocation]);

  const cardStyle: CSSProperties = {
    background: "var(--color-surface-variant)",
    borderRadius: 12,
    overflow: "hidden",
    marginBottom: 16,
    cursor: hasLocation ? "pointer" : "default",
  };

  const headerStyle: CSSProperties = {
    padding: 12,
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  };

  const statusDotStyle: CSSProperties = {
    width: 8,
    height: 8,
    borderRadius: "50%",
    backgroundColor: isLive ? "#34C759" : "var(--color-outline)",
    marginRight: 6,
  };

  // Hide the card entirely when there's no location data
  if (!hasLocation) {
    return null;
  }

  const lat = friend!.latitude!;
  const lng = friend!.longitude!;
  const tileUrl = isDarkTheme()
    ? "https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
    : "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";

  return (
    <div style={cardStyle} onClick={onExpand}>
      {/* Header info */}
      <div style={headerStyle}>
        <div style={{ flex: 1 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              marginBottom: 4,
            }}
          >
            <div style={statusDotStyle} />
            <span
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: "var(--color-on-surface)",
              }}
            >
              {isLive ? "Live" : "Location"}
            </span>
          </div>
          <div
            style={{
              fontSize: 11,
              color: "var(--color-on-surface-variant)",
              marginBottom: 2,
            }}
          >
            {locationText}
          </div>
          {friend!.last_updated && (
            <div
              style={{
                fontSize: 10,
                color: "var(--color-on-surface-variant)",
                opacity: 0.7,
              }}
            >
              Updated {formatLocationTime(friend!.last_updated)}
            </div>
          )}
        </div>
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          style={{ opacity: 0.5 }}
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </div>

      {/* Map preview */}
      <div
        style={{
          height: 100,
          width: "100%",
          position: "relative",
        }}
      >
        <MapContainer
          center={[lat, lng]}
          zoom={13}
          style={{ height: "100%", width: "100%", borderRadius: 0 }}
          zoomControl={false}
          attributionControl={false}
          dragging={false}
          scrollWheelZoom={false}
          doubleClickZoom={false}
          touchZoom={false}
          whenReady={() => setMapReady(true)}
        >
          <TileLayer url={tileUrl} />
          {mapReady && (
            <Marker
              position={[lat, lng]}
              icon={createAvatarMarkerIcon(contactName, contactAddress)}
            />
          )}
        </MapContainer>
      </div>
    </div>
  );
}
