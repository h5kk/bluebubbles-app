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
import { useSettingsStore } from "@/store/settingsStore";
import { useMessageStore } from "@/store/messageStore";
import { tauriSyncFull, tauriSyncMessages } from "@/hooks/useTauri";

interface SidebarProps {
  width?: number;
  children: React.ReactNode;
}

export function Sidebar({ width = 315, children }: SidebarProps) {
  const navigate = useNavigate();
  const { searchQuery, setSearchQuery, refreshChats, isRefreshing, lastRefreshTime } = useChatStore();
  const { status, error, errorAt } = useConnectionStore();
  const { demoMode, debugMode, updateSetting, themeMode, setThemeMode } = useSettingsStore();
  const messageCount = useMessageStore((s) => s.messages.length);
  const activeChatGuid = useMessageStore((s) => s.chatGuid);
  const [menuOpen, setMenuOpen] = useState(false);
  const [debugMenuOpen, setDebugMenuOpen] = useState(false);
  const chats = useChatStore((s) => s.chats);

  // Determine if current theme is dark
  const isDark = themeMode === "dark" ||
    (themeMode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  const wrapperStyle: CSSProperties = {
    width,
    minWidth: width,
    height: "100%",
    boxSizing: "border-box",
    padding: "8px 0 8px 8px",
    display: "flex",
  };

  const sidebarStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    backgroundColor: "var(--color-background)",
    border: "1px solid var(--color-surface-variant)",
    overflow: "hidden",
    borderRadius: 12,
    boxShadow: "var(--elevation-2)",
    position: "relative",
    zIndex: 2,
  };

  const topRowStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "10px 16px 6px",
    flexShrink: 0,
  };

  const handleCopyDiagnostics = useCallback(async () => {
    const rootStyles = getComputedStyle(document.documentElement);
    const textInput = document.querySelector<HTMLTextAreaElement>('textarea[aria-label="Message input"]');
    const inputWrapper = textInput?.parentElement;
    const inputBar = document.querySelector<HTMLElement>("[data-input-bar]");
    const chatScroll = document.querySelector<HTMLElement>("[data-chat-scroll]");
    const convoView = document.querySelector<HTMLElement>("[data-conversation-view]");
    const contentArea = document.querySelector<HTMLElement>("[data-app-content]");

    const inputRect = textInput?.getBoundingClientRect();
    const wrapperRect = inputWrapper?.getBoundingClientRect();
    const inputBarRect = inputBar?.getBoundingClientRect();
    const chatScrollRect = chatScroll?.getBoundingClientRect();
    const convoRect = convoView?.getBoundingClientRect();
    const contentRect = contentArea?.getBoundingClientRect();
    const inputStyles = textInput ? getComputedStyle(textInput) : null;
    const convoStyles = convoView ? getComputedStyle(convoView) : null;

    const lines = [
      `BlueBubbles Layout Diagnostics`,
      `Timestamp: ${new Date().toISOString()}`,
      ``,
      `Window: ${window.innerWidth}x${window.innerHeight} @${window.devicePixelRatio}x`,
      `Theme Mode: ${themeMode}`,
      `Theme: light=${useSettingsStore.getState().selectedLightTheme} dark=${useSettingsStore.getState().selectedDarkTheme}`,
      `Skin: ${useSettingsStore.getState().skin}`,
      `Active Chat: ${activeChatGuid ?? "none"}`,
      `Messages (loaded): ${messageCount}`,
      `Chats (loaded): ${chats.length}`,
      ``,
      `Input:`,
      `  textarea: ${inputRect ? `${Math.round(inputRect.width)}x${Math.round(inputRect.height)} (x:${Math.round(inputRect.x)} y:${Math.round(inputRect.y)})` : "not found"}`,
      `  wrapper: ${wrapperRect ? `${Math.round(wrapperRect.width)}x${Math.round(wrapperRect.height)} (x:${Math.round(wrapperRect.x)} y:${Math.round(wrapperRect.y)})` : "not found"}`,
      `  input bar: ${inputBarRect ? `${Math.round(inputBarRect.width)}x${Math.round(inputBarRect.height)} (x:${Math.round(inputBarRect.x)} y:${Math.round(inputBarRect.y)})` : "not found"}`,
      `  font-size: ${inputStyles?.fontSize ?? "n/a"}`,
      `  line-height: ${inputStyles?.lineHeight ?? "n/a"}`,
      `  padding: ${inputStyles?.padding ?? "n/a"}`,
      ``,
      `Chat Scroll:`,
      `  viewport: ${chatScrollRect ? `${Math.round(chatScrollRect.width)}x${Math.round(chatScrollRect.height)} (x:${Math.round(chatScrollRect.x)} y:${Math.round(chatScrollRect.y)})` : "not found"}`,
      `  clientHeight: ${chatScroll ? Math.round(chatScroll.clientHeight) : "n/a"}`,
      `  offsetHeight: ${chatScroll ? Math.round(chatScroll.offsetHeight) : "n/a"}`,
      `  scrollTop: ${chatScroll ? Math.round(chatScroll.scrollTop) : "n/a"}`,
      `  scrollHeight: ${chatScroll ? Math.round(chatScroll.scrollHeight) : "n/a"}`,
      ``,
      `Conversation View:`,
      `  bounds: ${convoRect ? `${Math.round(convoRect.width)}x${Math.round(convoRect.height)} (x:${Math.round(convoRect.x)} y:${Math.round(convoRect.y)})` : "not found"}`,
      `  gridTemplateRows: ${convoStyles?.gridTemplateRows ?? "n/a"}`,
      ``,
      `Content Area:`,
      `  bounds: ${contentRect ? `${Math.round(contentRect.width)}x${Math.round(contentRect.height)} (x:${Math.round(contentRect.x)} y:${Math.round(contentRect.y)})` : "not found"}`,
      ``,
      `Bubble Tokens:`,
      `  --bubble-padding-h: ${rootStyles.getPropertyValue("--bubble-padding-h").trim()}`,
      `  --bubble-padding-v: ${rootStyles.getPropertyValue("--bubble-padding-v").trim()}`,
      `  --bubble-radius-large: ${rootStyles.getPropertyValue("--bubble-radius-large").trim()}`,
      `  --bubble-radius-small: ${rootStyles.getPropertyValue("--bubble-radius-small").trim()}`,
      `  --bubble-max-width: ${rootStyles.getPropertyValue("--bubble-max-width").trim()}`,
      ``,
      `Layout Tokens:`,
      `  --title-bar-height: ${rootStyles.getPropertyValue("--title-bar-height").trim()}`,
      `  --sidebar-width: ${rootStyles.getPropertyValue("--sidebar-width").trim()}`,
    ];

    try {
      await navigator.clipboard.writeText(lines.join("\n"));
    } catch {
      // ignore clipboard errors
    }
  }, [activeChatGuid, chats.length, messageCount, themeMode]);

  return (
    <div style={wrapperStyle}>
      <div style={sidebarStyle} className="glass-panel">
      {/* Top row: hamburger left, refresh + compose right */}
      <div style={topRowStyle}>
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
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

          <SidebarIconButton
            label={isDark ? "Switch to light mode" : "Switch to dark mode"}
            onClick={() => {
              if (themeMode === "light") setThemeMode("dark");
              else if (themeMode === "dark") setThemeMode("light");
              else setThemeMode(isDark ? "light" : "dark");
            }}
          >
            {/* Sun/Moon icon */}
            {isDark ? (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <circle cx="8" cy="8" r="4" fill="currentColor" />
                <path d="M8 1V2M8 14V15M15 8H14M2 8H1M12.5 3.5L11.8 4.2M4.2 11.8L3.5 12.5M12.5 12.5L11.8 11.8M4.2 4.2L3.5 3.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M14 8.5C13.3 11.5 10.5 13.5 7.5 13C4.5 12.5 2.5 9.7 3 6.7C3.3 5 4.5 3.6 6 2.8C6.2 2.7 6.3 2.9 6.2 3.1C5.8 4.2 5.8 5.5 6.3 6.7C7.3 9 9.7 10.2 12 9.5C12.2 9.4 12.4 9.6 12.3 9.8C11.9 10.5 11.3 11.1 10.6 11.5" fill="currentColor" />
              </svg>
            )}
          </SidebarIconButton>

          {debugMode && (
            <div style={{ position: "relative" }}>
              <SidebarIconButton
                label="Debug tools"
                onClick={() => setDebugMenuOpen((o) => !o)}
              >
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <path d="M6 1H10" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
                  <rect x="4" y="3" width="8" height="10" rx="3" stroke="currentColor" strokeWidth="1.4" />
                  <path d="M2 6H4M12 6H14M2 10H4M12 10H14" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
                </svg>
              </SidebarIconButton>

              {debugMenuOpen && (
                <div
                  className="glass-panel-elevated"
                  style={{
                    position: "absolute",
                    top: "100%",
                    left: 0,
                    marginTop: 6,
                    zIndex: 120,
                    background: "var(--color-surface)",
                    borderRadius: 12,
                    boxShadow: "0 4px 20px rgba(0,0,0,0.18)",
                    padding: "4px 0",
                    minWidth: 200,
                  }}
                >
                  <MenuDropdownItem
                    label="Copy Layout Diagnostics"
                    onClick={() => {
                      handleCopyDiagnostics();
                      setDebugMenuOpen(false);
                    }}
                  />
                </div>
              )}
            </div>
          )}
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <RefreshButton
            isRefreshing={isRefreshing}
            lastRefreshTime={lastRefreshTime}
            onRefresh={refreshChats}
          />

          {error && (
            <button
              onClick={() => navigate("/settings?panel=server")}
              title={`Connection issue${errorAt ? ` (${new Date(errorAt).toLocaleTimeString()})` : ""}\n${error}`}
              style={{
                width: 28,
                height: 28,
                borderRadius: "50%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                backgroundColor: "var(--color-error-container)",
                color: "var(--color-error)",
                border: "1px solid var(--color-error)",
                cursor: "pointer",
              }}
              aria-label="Connection issue"
            >
              <span style={{ fontSize: 12, fontWeight: 700 }}>!</span>
            </button>
          )}

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
          <div style={{ height: 1, backgroundColor: "var(--color-outline)", opacity: 0.2, margin: "4px 0" }} />
          <MenuToggleItem
            label="Demo Mode"
            checked={demoMode}
            onChange={(checked) => {
              updateSetting("demoMode", checked ? "true" : "false");
            }}
          />
          <MenuToggleItem
            label="Debug Mode"
            checked={debugMode}
            onChange={(checked) => {
              updateSetting("debugMode", checked ? "true" : "false");
              if (!checked) setDebugMenuOpen(false);
            }}
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

      {debugMenuOpen && (
        <div
          onClick={() => setDebugMenuOpen(false)}
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 119,
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
      <LoadingLine visible={isRefreshing} height={1} />

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
          alignItems: "center",
          justifyContent: "center",
          padding: "4px 6px",
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
      </button>

      {hovered && secondsAgo !== null && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            right: 0,
            marginTop: 6,
            padding: "2px 6px",
            borderRadius: 6,
            background: "var(--color-surface-variant)",
            color: "var(--color-on-surface-variant)",
            fontSize: 10,
            lineHeight: 1.2,
            whiteSpace: "nowrap",
            boxShadow: "var(--elevation-1)",
            pointerEvents: "none",
          }}
        >
          {formatTimeAgo(secondsAgo)}
        </div>
      )}

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

interface MenuToggleItemProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function MenuToggleItem({ label, checked, onChange }: MenuToggleItemProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={() => onChange(!checked)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        width: "100%",
        padding: "8px 16px",
        fontSize: 13,
        color: "var(--color-on-surface)",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        cursor: "pointer",
        transition: "background-color 80ms ease",
      }}
    >
      <span>{label}</span>
      <div
        style={{
          width: 42,
          height: 24,
          borderRadius: 12,
          backgroundColor: checked ? "#34C759" : "var(--color-outline)",
          transition: "background-color 150ms ease",
          position: "relative",
          opacity: checked ? 1 : 0.5,
        }}
      >
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 10,
            backgroundColor: "white",
            position: "absolute",
            top: 2,
            left: checked ? 20 : 2,
            transition: "left 150ms ease",
            boxShadow: "0 1px 3px rgba(0,0,0,0.2)",
          }}
        />
      </div>
    </button>
  );
}
