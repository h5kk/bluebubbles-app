/**
 * Sidebar layout component - macOS Messages style.
 * Contains the conversation list, search, and navigation controls.
 */
import { useState, useEffect, useRef, useCallback, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { SearchBar } from "@/components/SearchBar";
import { LoadingLine } from "@/components/LoadingLine";
import { useChatStore } from "@/store/chatStore";
import { useConnectionStore } from "@/store/connectionStore";
import { tauriSyncFull, tauriSyncMessages } from "@/hooks/useTauri";

interface SidebarProps {
  width?: number;
  children: React.ReactNode;
}

export function Sidebar({ width = 340, children }: SidebarProps) {
  const navigate = useNavigate();
  const { searchQuery, setSearchQuery, refreshChats, isRefreshing, lastRefreshTime } = useChatStore();
  const { status } = useConnectionStore();
  const [menuOpen, setMenuOpen] = useState(false);

  const sidebarStyle: CSSProperties = {
    width,
    minWidth: width,
    height: "100%",
    display: "flex",
    flexDirection: "column",
    backgroundColor: "var(--color-background)",
    borderRight: "1px solid var(--color-surface-variant)",
    overflow: "hidden",
    borderTopRightRadius: 10,
    borderBottomRightRadius: 10,
  };

  const topRowStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "10px 16px 6px",
    flexShrink: 0,
  };

  return (
    <div style={sidebarStyle} className="glass-panel">
      {/* Top row: hamburger left, refresh + compose right */}
      <div style={topRowStyle}>
        <SidebarIconButton
          label="Menu"
          onClick={() => setMenuOpen((o) => !o)}
        >
          {/* Hamburger icon - 3 horizontal lines */}
          <svg width="18" height="14" viewBox="0 0 18 14" fill="none">
            <rect y="0" width="18" height="2" rx="1" fill="currentColor" />
            <rect y="6" width="18" height="2" rx="1" fill="currentColor" />
            <rect y="12" width="18" height="2" rx="1" fill="currentColor" />
          </svg>
        </SidebarIconButton>

        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <RefreshButton
            isRefreshing={isRefreshing}
            lastRefreshTime={lastRefreshTime}
            onRefresh={refreshChats}
          />

          <SidebarIconButton
            label="New message"
            onClick={() => navigate("/new")}
          >
            {/* Compose icon - square with pencil */}
            <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
              <rect x="1" y="3" width="12" height="14" rx="2" stroke="currentColor" strokeWidth="1.6" fill="none" />
              <path d="M9 3L15.5 0.5L17.5 2.5L11 9L8.5 9.5L9 7Z" fill="currentColor" />
            </svg>
          </SidebarIconButton>
        </div>
      </div>

      {/* Dropdown menu */}
      {menuOpen && (
        <div
          className="glass-panel-elevated"
          style={{
            position: "absolute",
            top: 44,
            left: 8,
            zIndex: 100,
            background: "var(--color-surface)",
            borderRadius: 12,
            boxShadow: "0 4px 20px rgba(0,0,0,0.18)",
            padding: "4px 0",
            minWidth: 180,
          }}
        >
          <MenuDropdownItem
            label="Chats"
            onClick={() => { navigate("/"); setMenuOpen(false); }}
          />
          <MenuDropdownItem
            label="Settings"
            onClick={() => { navigate("/settings"); setMenuOpen(false); }}
          />
          <MenuDropdownItem
            label="Find My"
            onClick={() => { navigate("/findmy"); setMenuOpen(false); }}
          />
        </div>
      )}
      {menuOpen && (
        <div
          onClick={() => setMenuOpen(false)}
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 99,
          }}
        />
      )}

      {/* Always-visible search bar */}
      <div style={{ padding: "4px 12px 8px", flexShrink: 0 }}>
        <SearchBar
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="Search"
        />
      </div>

      {/* Blue loading progress line */}
      <LoadingLine visible={isRefreshing} />

      {/* Connection indicator - subtle */}
      {status !== "connected" && (
        <div
          onClick={() => navigate("/settings?panel=server")}
          style={{
            padding: "4px 16px",
            fontSize: 11,
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
            cursor: "pointer",
            opacity: 0.85,
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

/* ---- Refresh Button ---- */

interface RefreshButtonProps {
  isRefreshing: boolean;
  lastRefreshTime: number | null;
  onRefresh: () => Promise<void>;
}

function RefreshButton({ isRefreshing, lastRefreshTime, onRefresh }: RefreshButtonProps) {
  const [hovered, setHovered] = useState(false);
  const [popoverOpen, setPopoverOpen] = useState(false);
  const [secondsAgo, setSecondsAgo] = useState<number | null>(null);
  const longPressTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isLongPressRef = useRef(false);
  const buttonRef = useRef<HTMLButtonElement>(null);

  const { refreshChats } = useChatStore();

  // Update "Xs ago" every second
  useEffect(() => {
    if (lastRefreshTime === null) {
      setSecondsAgo(null);
      return;
    }

    const update = () => {
      const diff = Math.floor((Date.now() - lastRefreshTime) / 1000);
      setSecondsAgo(diff);
    };

    update();
    const interval = setInterval(update, 1000);
    return () => clearInterval(interval);
  }, [lastRefreshTime]);

  // Long press handling
  const handleMouseDown = useCallback(() => {
    isLongPressRef.current = false;
    longPressTimerRef.current = setTimeout(() => {
      isLongPressRef.current = true;
      setPopoverOpen(true);
    }, 300);
  }, []);

  const handleMouseUp = useCallback(() => {
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
    // If it was a short click (not long press), do quick refresh
    if (!isLongPressRef.current && !isRefreshing) {
      onRefresh();
    }
  }, [isRefreshing, onRefresh]);

  const handleMouseLeave = useCallback(() => {
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
    setHovered(false);
  }, []);

  // Popover actions
  const handleQuickRefresh = useCallback(async () => {
    setPopoverOpen(false);
    await refreshChats();
  }, [refreshChats]);

  const handleFullResync = useCallback(async () => {
    setPopoverOpen(false);
    try {
      await tauriSyncFull();
    } catch {
      // sync error handled silently
    }
    await refreshChats();
  }, [refreshChats]);

  const handleSyncMessages = useCallback(async () => {
    setPopoverOpen(false);
    try {
      await tauriSyncMessages(25);
    } catch {
      // sync error handled silently
    }
    await refreshChats();
  }, [refreshChats]);

  const formatTimeAgo = (secs: number): string => {
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.floor(secs / 60);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ago`;
  };

  return (
    <div style={{ position: "relative" }}>
      <button
        ref={buttonRef}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={handleMouseLeave}
        aria-label="Refresh"
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 1,
          padding: "2px 6px",
          borderRadius: 6,
          backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
          color: isRefreshing ? "#007AFF" : "var(--color-on-surface-variant)",
          cursor: "pointer",
          transition: "background-color 100ms ease",
          minWidth: 36,
          height: 36,
          userSelect: "none",
        }}
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          style={{ display: "block" }}
        >
          {/* Circular arrow refresh icon */}
          <path
            d="M13.5 8A5.5 5.5 0 1 1 8 2.5"
            stroke="currentColor"
            strokeWidth="1.6"
            strokeLinecap="round"
            fill="none"
          />
          <path
            d="M8 0.5L10.5 2.5L8 4.5"
            stroke="currentColor"
            strokeWidth="1.6"
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
          />
        </svg>

        {secondsAgo !== null && (
          <span
            style={{
              fontSize: 10,
              lineHeight: 1,
              color: "var(--color-on-surface-variant)",
              whiteSpace: "nowrap",
            }}
          >
            {formatTimeAgo(secondsAgo)}
          </span>
        )}
      </button>

      {/* Long press popover */}
      <AnimatePresence>
        {popoverOpen && (
          <>
            {/* Backdrop to close popover */}
            <div
              onClick={() => setPopoverOpen(false)}
              style={{
                position: "fixed",
                inset: 0,
                zIndex: 199,
              }}
            />
            <motion.div
              initial={{ opacity: 0, scale: 0.92, y: -4 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.92, y: -4 }}
              transition={{ duration: 0.15 }}
              style={{
                position: "absolute",
                top: "100%",
                right: 0,
                marginTop: 6,
                zIndex: 200,
                background: "var(--color-surface)",
                borderRadius: 12,
                boxShadow: "0 4px 20px rgba(0,0,0,0.18)",
                padding: "4px 0",
                minWidth: 180,
                overflow: "hidden",
              }}
            >
              <PopoverItem
                label="Quick Refresh"
                sublabel="Refresh chat list"
                onClick={handleQuickRefresh}
              />
              <PopoverItem
                label="Full Re-sync"
                sublabel="Re-sync all data from server"
                onClick={handleFullResync}
              />
              <PopoverItem
                label="Sync Messages"
                sublabel="Sync recent messages"
                onClick={handleSyncMessages}
              />
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}

/* ---- Popover Item ---- */

interface PopoverItemProps {
  label: string;
  sublabel: string;
  onClick: () => void;
}

function PopoverItem({ label, sublabel, onClick }: PopoverItemProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "block",
        width: "100%",
        textAlign: "left",
        padding: "8px 14px",
        cursor: "pointer",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        transition: "background-color 80ms ease",
      }}
    >
      <div style={{ fontSize: 13, fontWeight: 500, color: "var(--color-on-surface)" }}>
        {label}
      </div>
      <div style={{ fontSize: 11, color: "var(--color-outline)", marginTop: 1 }}>
        {sublabel}
      </div>
    </button>
  );
}

/* ---- Internal components ---- */

interface SidebarIconButtonProps {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
}

function SidebarIconButton({ children, label, onClick }: SidebarIconButtonProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      aria-label={label}
      style={{
        width: 30,
        height: 30,
        borderRadius: 6,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: "var(--color-on-surface-variant)",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        transition: "background-color 100ms ease",
        cursor: "pointer",
      }}
    >
      {children}
    </button>
  );
}

interface MenuDropdownItemProps {
  label: string;
  onClick: () => void;
}

function MenuDropdownItem({ label, onClick }: MenuDropdownItemProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "block",
        width: "100%",
        textAlign: "left",
        padding: "8px 16px",
        fontSize: 13,
        color: "var(--color-on-surface)",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        cursor: "pointer",
        transition: "background-color 80ms ease",
      }}
    >
      {label}
    </button>
  );
}
