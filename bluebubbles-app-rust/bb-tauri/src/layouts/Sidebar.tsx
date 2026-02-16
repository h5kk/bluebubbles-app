/**
 * Sidebar layout component.
 * Contains the conversation list, search, and navigation controls.
 * Implements the sidebar from spec 02-conversation-list.md.
 */
import { useCallback, useState, type CSSProperties } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { SearchBar } from "@/components/SearchBar";
import { useChatStore } from "@/store/chatStore";
import { useConnectionStore } from "@/store/connectionStore";

interface SidebarProps {
  width?: number;
  children: React.ReactNode;
}

export function Sidebar({ width = 340, children }: SidebarProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const { searchQuery, setSearchQuery } = useChatStore();
  const { status } = useConnectionStore();
  const [showSearch, setShowSearch] = useState(false);

  const isActive = useCallback(
    (path: string) => location.pathname.startsWith(path),
    [location.pathname]
  );

  const sidebarStyle: CSSProperties = {
    width,
    minWidth: width,
    height: "100%",
    display: "flex",
    flexDirection: "column",
    backgroundColor: "var(--color-background)",
    borderRight: "1px solid var(--color-surface-variant)",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "12px 16px",
    flexShrink: 0,
  };

  const navStyle: CSSProperties = {
    display: "flex",
    gap: 4,
    padding: "0 8px 8px",
    flexShrink: 0,
  };

  return (
    <div style={sidebarStyle}>
      {/* Header */}
      <div style={headerStyle}>
        <h1
          style={{
            fontSize: "var(--font-title-large)",
            fontWeight: 700,
            color: "var(--color-on-surface)",
          }}
        >
          Messages
        </h1>

        <div style={{ display: "flex", gap: 4 }}>
          {/* Search toggle */}
          <HeaderButton
            label="Search"
            onClick={() => setShowSearch((s) => !s)}
            active={showSearch}
          >
            {"\uD83D\uDD0D"}
          </HeaderButton>

          {/* New message */}
          <HeaderButton label="New message" onClick={() => navigate("/new")}>
            {"\u270F\uFE0F"}
          </HeaderButton>
        </div>
      </div>

      {/* Search bar */}
      <AnimatePresence>
        {showSearch && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.15 }}
            style={{ overflow: "hidden", flexShrink: 0 }}
          >
            <SearchBar
              value={searchQuery}
              onChange={setSearchQuery}
              placeholder="Search conversations"
              autoFocus
            />
            <div style={{ height: 8 }} />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Navigation tabs */}
      <div style={navStyle}>
        <NavTab
          label="Chats"
          active={isActive("/chat") || location.pathname === "/"}
          onClick={() => navigate("/")}
        />
        <NavTab
          label="Settings"
          active={isActive("/settings")}
          onClick={() => navigate("/settings")}
        />
        <NavTab
          label="Find My"
          active={isActive("/findmy")}
          onClick={() => navigate("/findmy")}
        />
      </div>

      {/* Connection indicator */}
      {status !== "connected" && (
        <div
          style={{
            padding: "6px 16px",
            fontSize: "var(--font-label-small)",
            color:
              status === "connecting"
                ? "var(--color-tertiary)"
                : "var(--color-error)",
            backgroundColor:
              status === "connecting"
                ? "var(--color-tertiary-container)"
                : "var(--color-error-container)",
            textAlign: "center",
            flexShrink: 0,
          }}
        >
          {status === "connecting"
            ? "Connecting..."
            : status === "error"
              ? "Connection error"
              : "Disconnected"}
        </div>
      )}

      {/* Scrollable content area */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          scrollBehavior: "smooth",
        }}
      >
        {children}
      </div>
    </div>
  );
}

interface NavTabProps {
  label: string;
  active: boolean;
  onClick: () => void;
}

function NavTab({ label, active, onClick }: NavTabProps) {
  const [hovered, setHovered] = useState(false);

  const style: CSSProperties = {
    padding: "6px 14px",
    borderRadius: 16,
    fontSize: "var(--font-label-large)",
    fontWeight: active ? 600 : 400,
    color: active ? "var(--color-on-primary)" : "var(--color-on-surface-variant)",
    backgroundColor: active
      ? "var(--color-primary)"
      : hovered
        ? "var(--color-surface-variant)"
        : "transparent",
    transition: "all 150ms ease",
    cursor: "pointer",
  };

  return (
    <button
      style={style}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {label}
    </button>
  );
}

interface HeaderButtonProps {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  active?: boolean;
}

function HeaderButton({ children, label, onClick, active = false }: HeaderButtonProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      aria-label={label}
      style={{
        width: 32,
        height: 32,
        borderRadius: "50%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: 14,
        backgroundColor: active
          ? "var(--color-primary-container)"
          : hovered
            ? "var(--color-surface-variant)"
            : "transparent",
        transition: "background-color 100ms ease",
        cursor: "pointer",
      }}
    >
      {children}
    </button>
  );
}
