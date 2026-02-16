/**
 * Avatar component with gradient backgrounds, initials fallback, and group layout.
 * Matches the spec from 07-shared-components.md section 1.
 */
import { useMemo, type CSSProperties } from "react";

/** 7 deterministic gradient palettes from spec 01. */
const AVATAR_PALETTES = [
  ["#FD678D", "#FF8AA8"], // Pink
  ["#6BCFF6", "#94DDFD"], // Blue
  ["#FEA21C", "#FEB854"], // Orange
  ["#5EDE79", "#8DE798"], // Green
  ["#FFCA1C", "#FCD752"], // Yellow
  ["#FF534D", "#FD726A"], // Red
  ["#A78DF3", "#BCABFC"], // Purple
];

const GRAY_PALETTE = ["#928E8E", "#686868"];

function hashAddress(address: string): number {
  let total = 0;
  for (let i = 0; i < address.length; i++) {
    total += address.charCodeAt(i);
  }
  return total % 7;
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
  colorful?: boolean;
  avatarUrl?: string;
  style?: CSSProperties;
}

export function Avatar({
  name = "?",
  address = "",
  size = 40,
  colorful = true,
  avatarUrl,
  style,
}: AvatarProps) {
  const palette = useMemo(() => {
    if (!colorful) return GRAY_PALETTE;
    const seed = hashAddress(address || name);
    return AVATAR_PALETTES[seed] ?? AVATAR_PALETTES[0];
  }, [address, name, colorful]);

  const initials = useMemo(() => getInitials(name, true), [name]);

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
    background: avatarUrl
      ? "transparent"
      : `linear-gradient(135deg, ${palette[0]}, ${palette[1]})`,
    color: "#FFFFFF",
    fontSize: size * 0.4,
    fontWeight: 600,
    letterSpacing: "0.5px",
    userSelect: "none",
    flexShrink: 0,
    ...style,
  };

  if (avatarUrl) {
    return (
      <div style={containerStyle}>
        <img
          src={avatarUrl}
          alt={name}
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

  return <div style={containerStyle}>{initials}</div>;
}

/** Group avatar showing multiple participants. */
interface GroupAvatarProps {
  participants: Array<{ name: string; address: string; avatarUrl?: string }>;
  size?: number;
  colorful?: boolean;
}

export function GroupAvatar({
  participants,
  size = 40,
  colorful = true,
}: GroupAvatarProps) {
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
      }}
    >
      {display.map((p, i) => (
        <Avatar
          key={p.address + i}
          name={p.name}
          address={p.address}
          avatarUrl={p.avatarUrl}
          size={innerSize}
          colorful={colorful}
        />
      ))}
    </div>
  );
}
