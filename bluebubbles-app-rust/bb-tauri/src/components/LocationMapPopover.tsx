/**
 * LocationMapPopover - Full-screen map overlay showing contact's location.
 * Activated by clicking the LocationMapCard.
 */
import { useState, useMemo, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { MapContainer, TileLayer, Marker, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import type { FindMyFriend } from "@/store/findMyStore";
import { Avatar } from "./Avatar";
import { open as openShell } from "@tauri-apps/plugin-shell";

interface LocationMapPopoverProps {
  open: boolean;
  onClose: () => void;
  friend: FindMyFriend;
  contactName: string;
  contactAddress: string;
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

function createAvatarMarkerIcon(name: string): L.DivIcon {
  return L.divIcon({
    className: "",
    iconSize: [50, 50],
    iconAnchor: [25, 25],
    html: `
      <div style="
        width: 50px;
        height: 50px;
        border-radius: 50%;
        overflow: hidden;
        border: 4px solid #fff;
        box-shadow: 0 4px 12px rgba(0,0,0,0.4);
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-weight: 600;
        font-size: 20px;
      ">
        ${name.charAt(0).toUpperCase()}
      </div>
    `,
  });
}

function MapUpdater({ center, zoom }: { center: [number, number]; zoom: number }) {
  const map = useMap();
  map.setView(center, zoom);
  return null;
}

export function LocationMapPopover({
  open,
  onClose,
  friend,
  contactName,
  contactAddress,
}: LocationMapPopoverProps) {
  const [mapReady, setMapReady] = useState(false);

  const lat = friend.latitude!;
  const lng = friend.longitude!;

  const isLive = useMemo(() => {
    if (!friend.status) return false;
    const status = friend.status.toLowerCase();
    return status === "live" || status === "sharing";
  }, [friend]);

  const locationText = useMemo(() => {
    if (friend.address) return friend.address;
    return `${lat.toFixed(4)}, ${lng.toFixed(4)}`;
  }, [friend.address, lat, lng]);

  const handleOpenFindMy = async () => {
    try {
      await openShell("findmy://");
    } catch (err) {
      console.error("Failed to open Find My:", err);
    }
  };

  const tileUrl = isDarkTheme()
    ? "https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
    : "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";

  const overlayStyle: CSSProperties = {
    position: "fixed",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    zIndex: 9999,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "rgba(0, 0, 0, 0.6)",
  };

  const contentStyle: CSSProperties = {
    width: "90%",
    maxWidth: 800,
    height: "85%",
    maxHeight: 700,
    backgroundColor: "var(--color-surface)",
    borderRadius: 16,
    overflow: "hidden",
    display: "flex",
    flexDirection: "column",
    boxShadow: "0 20px 60px rgba(0,0,0,0.3)",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "16px 20px",
    borderBottom: "1px solid var(--color-surface-variant)",
    flexShrink: 0,
  };

  const infoStyle: CSSProperties = {
    padding: "16px 20px",
    borderBottom: "1px solid var(--color-surface-variant)",
    flexShrink: 0,
  };

  const statusDotStyle: CSSProperties = {
    width: 10,
    height: 10,
    borderRadius: "50%",
    backgroundColor: isLive ? "#34C759" : "var(--color-outline)",
    marginRight: 8,
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          style={overlayStyle}
          onClick={onClose}
        >
          <motion.div
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
            style={contentStyle}
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div style={headerStyle}>
              <h2
                style={{
                  fontSize: 18,
                  fontWeight: 600,
                  color: "var(--color-on-surface)",
                  margin: 0,
                }}
              >
                Location
              </h2>
              <button
                onClick={onClose}
                aria-label="Close"
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                  border: "none",
                  background: "var(--color-surface-variant)",
                  color: "var(--color-on-surface-variant)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  cursor: "pointer",
                  padding: 0,
                }}
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            {/* Contact info */}
            <div style={infoStyle}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  marginBottom: 12,
                }}
              >
                <Avatar name={contactName} address={contactAddress} size={40} />
                <div style={{ marginLeft: 12, flex: 1 }}>
                  <div
                    style={{
                      fontSize: 15,
                      fontWeight: 600,
                      color: "var(--color-on-surface)",
                      marginBottom: 2,
                    }}
                  >
                    {contactName}
                  </div>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                    }}
                  >
                    <div style={statusDotStyle} />
                    <span
                      style={{
                        fontSize: 13,
                        color: "var(--color-on-surface-variant)",
                      }}
                    >
                      {isLive ? "Live" : "Last known location"}
                    </span>
                  </div>
                </div>
              </div>

              <div
                style={{
                  fontSize: 13,
                  color: "var(--color-on-surface-variant)",
                  marginBottom: 4,
                }}
              >
                {locationText}
              </div>

              {friend.last_updated && (
                <div
                  style={{
                    fontSize: 12,
                    color: "var(--color-on-surface-variant)",
                    opacity: 0.7,
                  }}
                >
                  Updated {formatLocationTime(friend.last_updated)}
                </div>
              )}
            </div>

            {/* Map */}
            <div style={{ flex: 1, position: "relative" }}>
              <MapContainer
                center={[lat, lng]}
                zoom={14}
                style={{ height: "100%", width: "100%" }}
                zoomControl={true}
                attributionControl={false}
                whenReady={() => setMapReady(true)}
              >
                <TileLayer url={tileUrl} />
                {mapReady && (
                  <>
                    <MapUpdater center={[lat, lng]} zoom={14} />
                    <Marker
                      position={[lat, lng]}
                      icon={createAvatarMarkerIcon(contactName)}
                    />
                  </>
                )}
              </MapContainer>
            </div>

            {/* Footer with action buttons */}
            <div
              style={{
                padding: "16px 20px",
                borderTop: "1px solid var(--color-surface-variant)",
                display: "flex",
                gap: 12,
                flexShrink: 0,
              }}
            >
              <button
                onClick={handleOpenFindMy}
                style={{
                  flex: 1,
                  padding: "12px 20px",
                  borderRadius: 10,
                  border: "none",
                  background: "var(--color-primary)",
                  color: "white",
                  fontSize: 14,
                  fontWeight: 600,
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: 8,
                }}
              >
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <circle cx="12" cy="10" r="3" />
                  <path d="M12 21.7C17.3 17 20 13 20 10a8 8 0 1 0-16 0c0 3 2.7 6.9 8 11.7z" />
                </svg>
                Open in Find My
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
