/**
 * Avatar component styled after macOS Messages / iOS contacts.
 * Shows a person silhouette by default, with photo and initials as alternatives.
 * Loads contact avatars from the contact store (bulk-loaded) with a fallback
 * to individual IPC calls for addresses not yet in the store.
 */
import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { useContactStore } from "@/store/contactStore";
import { tauriGetContactAvatar } from "@/hooks/useTauri";

// Fallback cache for addresses resolved via individual IPC (before store is loaded)
const fallbackCache = new Map<string, string | null>();

/** Simple person silhouette SVG (head circle + shoulder arc), macOS Messages style. */
function PersonSilhouette({ size }: { size: number }) {
  const iconSize = size * 0.6;
  return (
    <svg
      width={iconSize}
      height={iconSize}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      {/* Head */}
      <circle cx="12" cy="8.5" r="4.5" fill="rgba(255,255,255,0.85)" />
      {/* Shoulders */}
      <path
        d="M3.5 22c0-4.7 3.8-8.5 8.5-8.5s8.5 3.8 8.5 8.5"
        fill="rgba(255,255,255,0.85)"
      />
    </svg>
  );
}

function getInitials(name: string, full = false): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 0) return "?";
  if (!full || parts.length === 1) {
    return (parts[0]?.[0] ?? "?").toUpperCase();
  }
  const first = parts[0]?.[0] ?? "";
  const last = parts[parts.length - 1]?.[0] ?? "";
  return (first + last).toUpperCase();
}

interface AvatarProps {
  name?: string;
  address?: string;
  size?: number;
  avatarUrl?: string;
  /** When true, show initials instead of the person silhouette as the no-photo fallback. */
  showInitials?: boolean;
  style?: CSSProperties;
}

export function Avatar({
  name = "?",
  address = "",
  size = 40,
  avatarUrl: providedAvatarUrl,
  showInitials = false,
  style,
}: AvatarProps) {
  // Read from the bulk-loaded contact store
  const storeAvatar = useContactStore((s) => s.getAvatar(address));
  const storeLoaded = useContactStore((s) => s.loaded);

  const [fallbackAvatar, setFallbackAvatar] = useState<string | null>(
    fallbackCache.get(address) ?? null
  );
  const [imgFailed, setImgFailed] = useState(false);

  // Fallback: if store is loaded but has no avatar for this address,
  // and we have not tried individual fetch yet, try one IPC call.
  // This handles edge cases where the store's address normalization
  // did not match but the Rust-side matching would.
  useEffect(() => {
    if (providedAvatarUrl || !address) return;
    // If the store already has an avatar, no need for fallback
    if (storeAvatar) return;
    // If store has not loaded yet, wait for it
    if (!storeLoaded) return;
    // If we already tried fallback for this address, skip
    if (fallbackCache.has(address)) {
      setFallbackAvatar(fallbackCache.get(address) ?? null);
      return;
    }

    let cancelled = false;
    tauriGetContactAvatar(address)
      .then((dataUri) => {
        if (!cancelled) {
          fallbackCache.set(address, dataUri);
          setFallbackAvatar(dataUri);
        }
      })
      .catch(() => {
        fallbackCache.set(address, null);
      });

    return () => {
      cancelled = true;
    };
  }, [address, providedAvatarUrl, storeAvatar, storeLoaded]);

  const avatarUrl = providedAvatarUrl || storeAvatar || fallbackAvatar;
  const initials = useMemo(() => getInitials(name, !showInitials), [name, showInitials]);
  const showImage = avatarUrl && !imgFailed;

  // Debug: log avatar resolution for first few renders
  useEffect(() => {
    if (address && !providedAvatarUrl) {
      console.debug(`[Avatar] "${name}" addr="${address}" storeLoaded=${storeLoaded} storeAvatar=${!!storeAvatar} fallback=${!!fallbackAvatar} showImage=${!!showImage}`);
    }
  }, [address, storeLoaded, storeAvatar, fallbackAvatar, showImage]);

  // macOS Messages style muted gray-blue gradient
  // Uses CSS variables for theme awareness with fallbacks
  const silhouetteBg = "linear-gradient(180deg, var(--avatar-bg-from, #A8B5C8), var(--avatar-bg-to, #8E9BB0))";
  const initialsBg = "linear-gradient(180deg, #6B7DB3, #5A6A9E)";

  const background = showImage
    ? "transparent"
    : showInitials
      ? initialsBg
      : silhouetteBg;

  const containerStyle: CSSProperties = {
    width: size,
    height: size,
    minWidth: size,
    minHeight: size,
    borderRadius: "50%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    overflow: "hidden",
    background,
    boxShadow: "inset 0 0 0 1px rgba(0,0,0,0.08)",
    border: showImage ? "1px solid rgba(0,0,0,0.1)" : "none",
    color: "#FFFFFF",
    fontSize: size * 0.38,
    fontWeight: 600,
    letterSpacing: "0.5px",
    userSelect: "none",
    flexShrink: 0,
    ...style,
  };

  if (showImage) {
    return (
      <div style={containerStyle}>
        <img
          src={avatarUrl}
          alt={name}
          onError={() => setImgFailed(true)}
          style={{
            width: "100%",
            height: "100%",
            objectFit: "cover",
            borderRadius: "50%",
          }}
        />
      </div>
    );
  }

  if (showInitials) {
    return <div style={containerStyle}>{initials}</div>;
  }

  return (
    <div style={containerStyle}>
      <PersonSilhouette size={size} />
    </div>
  );
}

/** Group avatar showing multiple participants. */
interface GroupAvatarProps {
  participants: Array<{ name: string; address: string; avatarUrl?: string }>;
  size?: number;
}

export function GroupAvatar({ participants, size = 40 }: GroupAvatarProps) {
  const display = participants.slice(0, 4);
  const innerSize = display.length <= 2 ? size * 0.55 : size * 0.45;

  return (
    <div
      style={{
        width: size,
        height: size,
        minWidth: size,
        minHeight: size,
        position: "relative",
        borderRadius: "50%",
        overflow: "hidden",
        display: "flex",
        flexWrap: "wrap",
        alignItems: "center",
        justifyContent: "center",
        gap: 1,
        background: "var(--color-surface-variant)",
        boxShadow: "inset 0 0 0 1px rgba(0,0,0,0.08)",
      }}
    >
      {display.map((p, i) => (
        <Avatar
          key={p.address + i}
          name={p.name}
          address={p.address}
          avatarUrl={p.avatarUrl}
          size={innerSize}
          showInitials
        />
      ))}
    </div>
  );
}
