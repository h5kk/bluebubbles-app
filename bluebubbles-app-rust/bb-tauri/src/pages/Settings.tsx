/**
 * Settings page - application settings hub.
 * Implements spec 04-settings-screens.md.
 */
import { useState, useCallback, useEffect, type CSSProperties, type ReactNode } from "react";
import { useSearchParams } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { useSettingsStore } from "@/store/settingsStore";
import { useConnectionStore } from "@/store/connectionStore";
import { useTheme, type ThemeMode } from "@/hooks/useTheme";
import {
  tauriConnect,
  tauriSyncFull,
  tauriSyncMessages,
  tauriCheckPrivateApiStatus,
  type PrivateApiStatus,
} from "@/hooks/useTauri";

type SettingsPanel =
  | "general"
  | "appearance"
  | "chat"
  | "notifications"
  | "private-api"
  | "desktop"
  | "server"
  | "diagnostics"
  | "troubleshoot"
  | "about";

const validPanels: SettingsPanel[] = [
  "general", "appearance", "chat", "notifications",
  "private-api", "desktop", "server", "diagnostics", "troubleshoot", "about",
];

export function Settings() {
  const [searchParams] = useSearchParams();
  const panelParam = searchParams.get("panel");
  const initialPanel = validPanels.includes(panelParam as SettingsPanel) ? (panelParam as SettingsPanel) : "general";
  const [activePanel, setActivePanel] = useState<SettingsPanel>(initialPanel);

  useEffect(() => {
    if (panelParam && validPanels.includes(panelParam as SettingsPanel)) {
      setActivePanel(panelParam as SettingsPanel);
    }
  }, [panelParam]);

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    padding: "16px 24px",
    borderBottom: "1px solid var(--color-surface-variant)",
    flexShrink: 0,
  };

  const bodyStyle: CSSProperties = {
    display: "flex",
    flex: 1,
    overflow: "hidden",
  };

  const navStyle: CSSProperties = {
    width: 200,
    borderRight: "1px solid var(--color-surface-variant)",
    padding: "8px 0",
    overflow: "auto",
    flexShrink: 0,
  };

  const contentStyle: CSSProperties = {
    flex: 1,
    overflow: "auto",
    padding: 24,
  };

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
          Settings
        </h1>
      </div>

      <div style={bodyStyle}>
        {/* Navigation */}
        <div style={navStyle}>
          <SettingsNavItem
            label="General"
            icon={"\u2699\uFE0F"}
            active={activePanel === "general"}
            onClick={() => setActivePanel("general")}
          />
          <SettingsNavItem
            label="Appearance"
            icon={"\uD83C\uDFA8"}
            active={activePanel === "appearance"}
            onClick={() => setActivePanel("appearance")}
          />
          <SettingsNavItem
            label="Chat"
            icon={"\uD83D\uDCAC"}
            active={activePanel === "chat"}
            onClick={() => setActivePanel("chat")}
          />
          <SettingsNavItem
            label="Notifications"
            icon={"\uD83D\uDD14"}
            active={activePanel === "notifications"}
            onClick={() => setActivePanel("notifications")}
          />
          <SettingsNavItem
            label="Private API"
            icon={"\uD83D\uDD12"}
            active={activePanel === "private-api"}
            onClick={() => setActivePanel("private-api")}
          />
          <SettingsNavItem
            label="Desktop"
            icon={"\uD83D\uDDA5\uFE0F"}
            active={activePanel === "desktop"}
            onClick={() => setActivePanel("desktop")}
          />
          <SettingsNavItem
            label="Server"
            icon={"\uD83C\uDF10"}
            active={activePanel === "server"}
            onClick={() => setActivePanel("server")}
          />
          <SettingsNavItem
            label="Diagnostics"
            icon={"\uD83D\uDCCA"}
            active={activePanel === "diagnostics"}
            onClick={() => setActivePanel("diagnostics")}
          />
          <SettingsNavItem
            label="Troubleshoot"
            icon={"\uD83D\uDEE0\uFE0F"}
            active={activePanel === "troubleshoot"}
            onClick={() => setActivePanel("troubleshoot")}
          />
          <SettingsNavItem
            label="About"
            icon={"\u2139\uFE0F"}
            active={activePanel === "about"}
            onClick={() => setActivePanel("about")}
          />
        </div>

        {/* Panel content */}
        <div style={contentStyle}>
          <AnimatePresence mode="wait">
            <motion.div
              key={activePanel}
              initial={{ opacity: 0, x: 10 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -10 }}
              transition={{ duration: 0.15 }}
            >
              {activePanel === "general" && <GeneralPanel />}
              {activePanel === "appearance" && <AppearancePanel />}
              {activePanel === "chat" && <ChatPanel />}
              {activePanel === "notifications" && <NotificationsPanel />}
              {activePanel === "private-api" && <PrivateAPIPanel />}
              {activePanel === "desktop" && <DesktopPanel />}
              {activePanel === "server" && <ServerPanel />}
              {activePanel === "diagnostics" && <DiagnosticsPanel />}
              {activePanel === "troubleshoot" && <TroubleshootPanel />}
              {activePanel === "about" && <AboutPanel />}
            </motion.div>
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}

// ─── Navigation Item ──────────────────────────────────────────────────────────

interface SettingsNavItemProps {
  label: string;
  icon: string;
  active: boolean;
  onClick: () => void;
}

function SettingsNavItem({ label, icon, active, onClick }: SettingsNavItemProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: "100%",
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "10px 16px",
        fontSize: "var(--font-body-medium)",
        fontWeight: active ? 600 : 400,
        color: active ? "var(--color-on-primary-container)" : "var(--color-on-surface)",
        backgroundColor: active
          ? "var(--color-primary-container)"
          : hovered
            ? "var(--color-surface-variant)"
            : "transparent",
        transition: "all 100ms ease",
        cursor: "pointer",
        textAlign: "left",
      }}
    >
      <span style={{ fontSize: 16 }}>{icon}</span>
      {label}
    </button>
  );
}

// ─── Setting Widgets ──────────────────────────────────────────────────────────

function SettingsSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div style={{ marginBottom: 32 }}>
      <h3
        style={{
          fontSize: "var(--font-body-large)",
          fontWeight: 600,
          color: "var(--color-on-surface)",
          marginBottom: 12,
          paddingBottom: 8,
          borderBottom: "1px solid var(--color-surface-variant)",
        }}
      >
        {title}
      </h3>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>{children}</div>
    </div>
  );
}

function SettingsTile({
  label,
  subtitle,
  trailing,
  onClick,
}: {
  label: string;
  subtitle?: string;
  trailing?: ReactNode;
  onClick?: () => void;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <div
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "10px 0",
        gap: 16,
        cursor: onClick ? "pointer" : "default",
        borderRadius: onClick ? 8 : 0,
        backgroundColor: onClick && hovered ? "var(--color-surface-variant)" : "transparent",
        transition: "background-color 100ms ease",
      }}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface)",
          }}
        >
          {label}
        </div>
        {subtitle && (
          <div
            style={{
              fontSize: "var(--font-body-small)",
              color: "var(--color-on-surface-variant)",
              marginTop: 2,
            }}
          >
            {subtitle}
          </div>
        )}
      </div>
      {trailing && <div style={{ flexShrink: 0 }}>{trailing}</div>}
    </div>
  );
}

function SettingsSwitch({
  label,
  subtitle,
  value,
  onChange,
  disabled,
}: {
  label: string;
  subtitle?: string;
  value: boolean;
  onChange: (val: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <SettingsTile
      label={label}
      subtitle={subtitle}
      trailing={
        <button
          onClick={() => !disabled && onChange(!value)}
          style={{
            width: 44,
            height: 24,
            borderRadius: 12,
            backgroundColor: disabled
              ? "var(--color-surface-variant)"
              : value
                ? "var(--color-primary)"
                : "var(--color-outline)",
            position: "relative",
            transition: "background-color 200ms ease",
            cursor: disabled ? "not-allowed" : "pointer",
            opacity: disabled ? 0.5 : 1,
          }}
          role="switch"
          aria-checked={value}
          disabled={disabled}
        >
          <motion.div
            animate={{ x: value ? 20 : 0 }}
            transition={{ duration: 0.15 }}
            style={{
              width: 20,
              height: 20,
              borderRadius: "50%",
              backgroundColor: "#FFFFFF",
              position: "absolute",
              top: 2,
              left: 2,
              boxShadow: "0 1px 3px rgba(0,0,0,0.3)",
            }}
          />
        </button>
      }
    />
  );
}

function SettingsDropdown({
  label,
  subtitle,
  value,
  options,
  onChange,
}: {
  label: string;
  subtitle?: string;
  value: string;
  options: Array<{ label: string; value: string }>;
  onChange: (val: string) => void;
}) {
  return (
    <SettingsTile
      label={label}
      subtitle={subtitle}
      trailing={
        <select
          value={value}
          onChange={(e) => onChange(e.target.value)}
          style={{
            padding: "6px 12px",
            borderRadius: 8,
            border: "1px solid var(--color-outline)",
            backgroundColor: "var(--color-surface-variant)",
            color: "var(--color-on-surface)",
            fontSize: "var(--font-body-medium)",
            cursor: "pointer",
          }}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      }
    />
  );
}

function SettingsButton({
  label,
  subtitle,
  buttonLabel,
  onClick,
  variant = "primary",
  loading,
}: {
  label: string;
  subtitle?: string;
  buttonLabel: string;
  onClick: () => void;
  variant?: "primary" | "error" | "outline";
  loading?: boolean;
}) {
  const bgMap = {
    primary: "var(--color-primary)",
    error: "var(--color-error)",
    outline: "transparent",
  };
  const colorMap = {
    primary: "var(--color-on-primary)",
    error: "var(--color-on-error, #fff)",
    outline: "var(--color-primary)",
  };

  return (
    <SettingsTile
      label={label}
      subtitle={subtitle}
      trailing={
        <button
          onClick={onClick}
          disabled={loading}
          style={{
            padding: "6px 16px",
            borderRadius: 8,
            fontSize: "var(--font-body-small)",
            fontWeight: 600,
            cursor: loading ? "not-allowed" : "pointer",
            backgroundColor: bgMap[variant],
            color: colorMap[variant],
            border: variant === "outline" ? "1px solid var(--color-outline)" : "none",
            opacity: loading ? 0.6 : 1,
            transition: "opacity 150ms ease",
          }}
        >
          {loading ? "..." : buttonLabel}
        </button>
      }
    />
  );
}

// ─── Panels ───────────────────────────────────────────────────────────────────

function GeneralPanel() {
  const { sendWithReturn, tabletMode, updateSetting, settings } = useSettingsStore();
  const autoOpenKeyboard = settings["autoOpenKeyboard"] !== "false";
  const generateLinkPreviews = settings["generateLinkPreviews"] !== "false";

  return (
    <>
      <SettingsSection title="Input">
        <SettingsSwitch
          label="Send with Return"
          subtitle="Press Enter to send messages instead of adding a new line"
          value={sendWithReturn}
          onChange={(v) => updateSetting("sendWithReturn", String(v))}
        />
        <SettingsSwitch
          label="Auto-Open Keyboard"
          subtitle="Automatically focus the message input when opening a chat"
          value={autoOpenKeyboard}
          onChange={(v) => updateSetting("autoOpenKeyboard", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Layout">
        <SettingsSwitch
          label="Tablet Mode"
          subtitle="Show conversation list alongside the message view"
          value={tabletMode}
          onChange={(v) => updateSetting("tabletMode", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Links">
        <SettingsSwitch
          label="Generate Link Previews"
          subtitle="Show rich previews for URLs shared in messages"
          value={generateLinkPreviews}
          onChange={(v) => updateSetting("generateLinkPreviews", String(v))}
        />
      </SettingsSection>
    </>
  );
}

function AppearancePanel() {
  const { themeMode, setThemeMode } = useTheme();
  const {
    skin,
    colorfulAvatars,
    colorfulBubbles,
    selectedLightTheme,
    selectedDarkTheme,
    setSkin,
    updateSetting,
    settings,
  } = useSettingsStore();
  const denseConversationTiles = settings["denseConversationTiles"] === "true";
  const showAvatarsInDMs = settings["showAvatarsInDMs"] !== "false";

  return (
    <>
      <SettingsSection title="Theme">
        <SettingsDropdown
          label="Theme Mode"
          value={themeMode}
          options={[
            { label: "Light", value: "light" },
            { label: "Dark", value: "dark" },
            { label: "System", value: "system" },
          ]}
          onChange={(v) => setThemeMode(v as ThemeMode)}
        />
        <SettingsDropdown
          label="Light Theme"
          subtitle="Theme used in light mode"
          value={selectedLightTheme}
          options={[
            { label: "Bright White", value: "Bright White" },
            { label: "Blue Light", value: "Blue Light" },
            { label: "Pink Light", value: "Pink Light" },
            { label: "Liquid Glass", value: "Liquid Glass Light" },
          ]}
          onChange={(v) => updateSetting("selected-light", v)}
        />
        <SettingsDropdown
          label="Dark Theme"
          subtitle="Theme used in dark mode"
          value={selectedDarkTheme}
          options={[
            { label: "OLED Dark", value: "OLED Dark" },
            { label: "Blue Dark", value: "Blue Dark" },
            { label: "Indigo Dark", value: "Indigo Dark" },
            { label: "Nord", value: "Nord" },
            { label: "Green Dark", value: "Green Dark" },
            { label: "Purple Dark", value: "Purple Dark" },
            { label: "Liquid Glass", value: "Liquid Glass Dark" },
          ]}
          onChange={(v) => updateSetting("selected-dark", v)}
        />
      </SettingsSection>

      <SettingsSection title="Skin">
        <SettingsDropdown
          label="UI Skin"
          subtitle="Visual style for the interface"
          value={skin}
          options={[
            { label: "iOS", value: "ios" },
            { label: "Material", value: "material" },
            { label: "Samsung", value: "samsung" },
          ]}
          onChange={(v) => setSkin(v as "ios" | "material" | "samsung")}
        />
      </SettingsSection>

      <SettingsSection title="Avatars & Bubbles">
        <SettingsSwitch
          label="Colorful Avatars"
          subtitle="Use gradient colors for contact avatars"
          value={colorfulAvatars}
          onChange={(v) => updateSetting("colorfulAvatars", String(v))}
        />
        <SettingsSwitch
          label="Colorful Bubbles"
          subtitle="Use custom colors for message bubbles"
          value={colorfulBubbles}
          onChange={(v) => updateSetting("colorfulBubbles", String(v))}
        />
        <SettingsSwitch
          label="Show Avatars in DM Chats"
          subtitle="Display contact avatars in one-on-one conversations"
          value={showAvatarsInDMs}
          onChange={(v) => updateSetting("showAvatarsInDMs", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Chat List">
        <SettingsSwitch
          label="Dense Conversation Tiles"
          subtitle="Show more conversations by using compact tile layout"
          value={denseConversationTiles}
          onChange={(v) => updateSetting("denseConversationTiles", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Window">
        <SettingsDropdown
          label="Window Effect"
          subtitle="Visual effect applied to the window background (requires restart)"
          value={settings["windowEffect"] ?? "none"}
          options={[
            { label: "None", value: "none" },
            { label: "Transparent", value: "transparent" },
            { label: "Acrylic", value: "acrylic" },
            { label: "Mica", value: "mica" },
          ]}
          onChange={(v) => updateSetting("windowEffect", v)}
        />
      </SettingsSection>
    </>
  );
}

function ChatPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const showDeliveryReceipts = settings["showDeliveryReceipts"] === "true";
  const showTypingIndicators = settings["showTypingIndicators"] !== "false";
  const autoSaveMedia = settings["autoSaveMedia"] === "true";
  const autoDownloadAttachments = settings["autoDownloadAttachments"] !== "false";
  const showDeliveryTimestamps = settings["showDeliveryTimestamps"] === "true";
  const showRepliesToPrevious = settings["showRepliesToPrevious"] !== "false";
  const filterUnknownSenders = settings["filterUnknownSenders"] === "true";
  const unarchiveOnNewMessage = settings["unarchiveOnNewMessage"] !== "false";

  return (
    <>
      <SettingsSection title="Messages">
        <SettingsSwitch
          label="Show Delivery Receipts"
          subtitle="Display read and delivered indicators on sent messages"
          value={showDeliveryReceipts}
          onChange={(v) => updateSetting("showDeliveryReceipts", String(v))}
        />
        <SettingsSwitch
          label="Show Typing Indicators"
          subtitle="See when someone is typing a message"
          value={showTypingIndicators}
          onChange={(v) => updateSetting("showTypingIndicators", String(v))}
        />
        <SettingsSwitch
          label="Show Delivery Timestamps"
          subtitle="Display exact times for message delivery and read receipts"
          value={showDeliveryTimestamps}
          onChange={(v) => updateSetting("showDeliveryTimestamps", String(v))}
        />
        <SettingsSwitch
          label="Show Replies to Previous Message"
          subtitle="Display inline replies context in conversation"
          value={showRepliesToPrevious}
          onChange={(v) => updateSetting("showRepliesToPrevious", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Media">
        <SettingsSwitch
          label="Auto-Download Attachments"
          subtitle="Automatically download images, videos, and files"
          value={autoDownloadAttachments}
          onChange={(v) => updateSetting("autoDownloadAttachments", String(v))}
        />
        <SettingsSwitch
          label="Auto-Save Media"
          subtitle="Save downloaded media to your computer"
          value={autoSaveMedia}
          onChange={(v) => updateSetting("autoSaveMedia", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Chat List">
        <SettingsSwitch
          label="Filter Unknown Senders"
          subtitle="Separate messages from unknown contacts"
          value={filterUnknownSenders}
          onChange={(v) => updateSetting("filterUnknownSenders", String(v))}
        />
        <SettingsSwitch
          label="Unarchive on New Message"
          subtitle="Move archived chats back when new messages arrive"
          value={unarchiveOnNewMessage}
          onChange={(v) => updateSetting("unarchiveOnNewMessage", String(v))}
        />
      </SettingsSection>
    </>
  );
}

function NotificationsPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const notificationsEnabled = settings["notificationsEnabled"] !== "false";
  const soundEnabled = settings["soundEnabled"] !== "false";
  const showPreviews = settings["notifShowPreview"] !== "false";
  const notifyReactions = settings["notifyReactions"] !== "false";
  const notifOnChatList = settings["notifOnChatList"] === "true";

  return (
    <>
      <SettingsSection title="Notifications">
        <SettingsSwitch
          label="Enable Notifications"
          subtitle="Show desktop notifications for new messages"
          value={notificationsEnabled}
          onChange={(v) => updateSetting("notificationsEnabled", String(v))}
        />
        <SettingsSwitch
          label="Notification Sound"
          subtitle="Play a sound when a new message arrives"
          value={soundEnabled}
          onChange={(v) => updateSetting("soundEnabled", String(v))}
        />
        <SettingsSwitch
          label="Show Message Preview"
          subtitle="Display message content in notification"
          value={showPreviews}
          onChange={(v) => updateSetting("notifShowPreview", String(v))}
        />
        <SettingsSwitch
          label="Notify for Reactions"
          subtitle="Receive notifications when someone reacts to your messages"
          value={notifyReactions}
          onChange={(v) => updateSetting("notifyReactions", String(v))}
        />
        <SettingsSwitch
          label="Notify on Chat List"
          subtitle="Show notifications even when the chat list is visible"
          value={notifOnChatList}
          onChange={(v) => updateSetting("notifOnChatList", String(v))}
        />
      </SettingsSection>
    </>
  );
}

function PrivateAPIPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const { serverInfo, status } = useConnectionStore();
  const [liveStatus, setLiveStatus] = useState<PrivateApiStatus | null>(null);
  const [statusLoading, setStatusLoading] = useState(false);
  const [statusError, setStatusError] = useState<string | null>(null);

  const privateAPIEnabled = settings["enablePrivateAPI"] === "true";
  const sendTapbacks = settings["privateSendTapbacks"] !== "false";
  const sendTypingIndicators = settings["privateSendTyping"] !== "false";
  const sendReadReceipts = settings["privateSendRead"] !== "false";
  const sendWithEffects = settings["privateSendEffects"] !== "false";
  const editMessages = settings["privateEditMessages"] !== "false";
  const unsendMessages = settings["privateUnsendMessages"] !== "false";

  // Fetch live Private API status from the server when panel opens
  const fetchStatus = useCallback(async () => {
    if (status !== "connected") return;
    setStatusLoading(true);
    setStatusError(null);
    try {
      const result = await tauriCheckPrivateApiStatus();
      setLiveStatus(result);
    } catch (err: unknown) {
      setStatusError(err instanceof Error ? err.message : String(err));
    } finally {
      setStatusLoading(false);
    }
  }, [status]);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  // Use live status if available, fall back to cached serverInfo
  const serverSupportsPrivateAPI = liveStatus
    ? liveStatus.enabled
    : serverInfo?.private_api === true;

  const helperConnected = liveStatus
    ? liveStatus.helper_connected
    : serverInfo?.helper_connected === true;

  return (
    <>
      <SettingsSection title="Private API">
        {/* Live status display */}
        {status === "connected" && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              padding: "10px 16px",
              borderRadius: 8,
              backgroundColor: serverSupportsPrivateAPI
                ? "var(--color-primary-container)"
                : "var(--color-tertiary-container)",
              marginBottom: 12,
            }}
          >
            <div
              style={{
                width: 10,
                height: 10,
                borderRadius: "50%",
                backgroundColor: statusLoading
                  ? "var(--color-outline)"
                  : serverSupportsPrivateAPI && helperConnected
                    ? "#34C759"
                    : serverSupportsPrivateAPI
                      ? "#FF9500"
                      : "#FF3B30",
                flexShrink: 0,
              }}
            />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: "var(--font-body-medium)",
                  fontWeight: 600,
                  color: serverSupportsPrivateAPI
                    ? "var(--color-on-primary-container)"
                    : "var(--color-on-tertiary-container)",
                }}
              >
                {statusLoading
                  ? "Checking status..."
                  : serverSupportsPrivateAPI && helperConnected
                    ? "Private API Active"
                    : serverSupportsPrivateAPI
                      ? "Private API Enabled (Helper Disconnected)"
                      : "Private API Disabled"}
              </div>
              <div
                style={{
                  fontSize: "var(--font-body-small)",
                  color: serverSupportsPrivateAPI
                    ? "var(--color-on-primary-container)"
                    : "var(--color-on-tertiary-container)",
                  opacity: 0.8,
                  marginTop: 2,
                }}
              >
                {statusLoading
                  ? "Querying server..."
                  : statusError
                    ? `Error: ${statusError}`
                    : serverSupportsPrivateAPI && helperConnected
                      ? "All Private API features are available"
                      : serverSupportsPrivateAPI
                        ? "The helper process is not connected. Restart it on the server."
                        : "Enable it in your BlueBubbles server settings to unlock these features."}
              </div>
            </div>
            <button
              onClick={fetchStatus}
              disabled={statusLoading}
              style={{
                padding: "4px 10px",
                borderRadius: 6,
                fontSize: "var(--font-body-small)",
                fontWeight: 600,
                cursor: statusLoading ? "not-allowed" : "pointer",
                backgroundColor: "transparent",
                color: serverSupportsPrivateAPI
                  ? "var(--color-on-primary-container)"
                  : "var(--color-on-tertiary-container)",
                border: "1px solid currentColor",
                opacity: statusLoading ? 0.5 : 0.7,
              }}
            >
              Refresh
            </button>
          </div>
        )}
        <SettingsSwitch
          label="Enable Private API"
          subtitle="Unlock advanced iMessage features like tapbacks, effects, and read receipts"
          value={privateAPIEnabled}
          onChange={(v) => updateSetting("enablePrivateAPI", String(v))}
          disabled={!serverSupportsPrivateAPI}
        />
      </SettingsSection>

      {privateAPIEnabled && serverSupportsPrivateAPI && (
        <>
          <SettingsSection title="Features">
            <SettingsSwitch
              label="Send Tapbacks"
              subtitle="React to messages with thumbs up, heart, etc."
              value={sendTapbacks}
              onChange={(v) => updateSetting("privateSendTapbacks", String(v))}
            />
            <SettingsSwitch
              label="Send Typing Indicators"
              subtitle="Let others know when you're typing"
              value={sendTypingIndicators}
              onChange={(v) => updateSetting("privateSendTyping", String(v))}
            />
            <SettingsSwitch
              label="Send Read Receipts"
              subtitle="Mark messages as read on the server"
              value={sendReadReceipts}
              onChange={(v) => updateSetting("privateSendRead", String(v))}
            />
            <SettingsSwitch
              label="Send with Effects"
              subtitle="Send messages with bubble and screen effects"
              value={sendWithEffects}
              onChange={(v) => updateSetting("privateSendEffects", String(v))}
            />
            <SettingsSwitch
              label="Edit Messages"
              subtitle="Edit sent messages (requires macOS Ventura+)"
              value={editMessages}
              onChange={(v) => updateSetting("privateEditMessages", String(v))}
            />
            <SettingsSwitch
              label="Unsend Messages"
              subtitle="Recall sent messages (requires macOS Ventura+)"
              value={unsendMessages}
              onChange={(v) => updateSetting("privateUnsendMessages", String(v))}
            />
          </SettingsSection>
        </>
      )}
    </>
  );
}

function DesktopPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const minimizeToTray = settings["minimizeToTray"] === "true";
  const closeToTray = settings["closeToTray"] === "true";
  const launchAtStartup = settings["launchAtStartup"] === "true";
  const startupPage = settings["startupPage"] ?? "conversations";
  return (
    <>
      <SettingsSection title="Window">
        <SettingsSwitch
          label="Minimize to System Tray"
          subtitle="Keep running in the tray when minimized"
          value={minimizeToTray}
          onChange={(v) => updateSetting("minimizeToTray", String(v))}
        />
        <SettingsSwitch
          label="Close to System Tray"
          subtitle="Keep running in the tray when closed"
          value={closeToTray}
          onChange={(v) => updateSetting("closeToTray", String(v))}
        />
      </SettingsSection>

      <SettingsSection title="Startup">
        <SettingsSwitch
          label="Launch at Startup"
          subtitle="Start BlueBubbles when your computer starts"
          value={launchAtStartup}
          onChange={(v) => updateSetting("launchAtStartup", String(v))}
        />
        <SettingsDropdown
          label="Startup Page"
          subtitle="Page shown when the app starts"
          value={startupPage}
          options={[
            { label: "Conversation List", value: "conversations" },
            { label: "Last Chat", value: "last-chat" },
          ]}
          onChange={(v) => updateSetting("startupPage", v)}
        />
      </SettingsSection>
    </>
  );
}

function ServerPanel() {
  const { status, serverInfo, setStatus, setServerInfo, setError, reset } = useConnectionStore();
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [connectError, setConnectError] = useState<string | null>(null);

  const isConnected = status === "connected";
  const isConnecting = status === "connecting";

  const handleConnect = useCallback(async () => {
    if (!address.trim() || !password.trim()) return;
    setConnectError(null);
    setStatus("connecting");
    try {
      const info = await tauriConnect(address.trim(), password.trim());
      setServerInfo(info);
      setStatus("connected");
      setError(null);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setConnectError(msg);
      setStatus("error");
      setError(msg);
    }
  }, [address, password, setStatus, setServerInfo, setError]);

  const handleDisconnect = useCallback(() => {
    reset();
  }, [reset]);

  const inputStyle: CSSProperties = {
    width: "100%",
    padding: "8px 12px",
    borderRadius: 8,
    border: "1px solid var(--color-outline)",
    backgroundColor: "var(--color-surface-variant)",
    color: "var(--color-on-surface)",
    fontSize: "var(--font-body-medium)",
    outline: "none",
  };

  const buttonStyle: CSSProperties = {
    padding: "8px 20px",
    borderRadius: 8,
    fontSize: "var(--font-body-medium)",
    fontWeight: 600,
    cursor: isConnecting ? "not-allowed" : "pointer",
    transition: "background-color 150ms ease",
  };

  return (
    <>
      <SettingsSection title="Connection">
        {!isConnected ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div>
              <label
                style={{
                  display: "block",
                  fontSize: "var(--font-body-medium)",
                  color: "var(--color-on-surface)",
                  marginBottom: 4,
                }}
              >
                Server Address
              </label>
              <input
                type="text"
                placeholder="http://192.168.1.100:1234"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                disabled={isConnecting}
                style={inputStyle}
              />
            </div>
            <div>
              <label
                style={{
                  display: "block",
                  fontSize: "var(--font-body-medium)",
                  color: "var(--color-on-surface)",
                  marginBottom: 4,
                }}
              >
                Password
              </label>
              <input
                type="password"
                placeholder="Server password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={isConnecting}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleConnect();
                }}
                style={inputStyle}
              />
            </div>
            {connectError && (
              <div
                style={{
                  fontSize: "var(--font-body-small)",
                  color: "var(--color-error)",
                  padding: "6px 10px",
                  borderRadius: 6,
                  backgroundColor: "var(--color-error-container)",
                }}
              >
                {connectError}
              </div>
            )}
            <button
              onClick={handleConnect}
              disabled={isConnecting || !address.trim() || !password.trim()}
              style={{
                ...buttonStyle,
                backgroundColor: "var(--color-primary)",
                color: "var(--color-on-primary)",
                opacity: isConnecting || !address.trim() || !password.trim() ? 0.6 : 1,
                alignSelf: "flex-start",
              }}
            >
              {isConnecting ? "Connecting..." : "Connect"}
            </button>
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <div
              style={{
                fontSize: "var(--font-body-medium)",
                color: "var(--color-primary)",
                fontWeight: 600,
              }}
            >
              Connected
            </div>
            <button
              onClick={handleDisconnect}
              style={{
                ...buttonStyle,
                backgroundColor: "var(--color-error)",
                color: "var(--color-on-error, #fff)",
                alignSelf: "flex-start",
              }}
            >
              Disconnect
            </button>
          </div>
        )}
      </SettingsSection>

      {isConnected && (
        <SettingsSection title="Server Information">
          <SettingsTile
            label="Server Version"
            trailing={
              <span style={{ color: "var(--color-on-surface-variant)" }}>
                {serverInfo?.server_version ?? "Unknown"}
              </span>
            }
          />
          <SettingsTile
            label="macOS Version"
            trailing={
              <span style={{ color: "var(--color-on-surface-variant)" }}>
                {serverInfo?.os_version ?? "Unknown"}
              </span>
            }
          />
          <SettingsTile
            label="Private API"
            trailing={
              <span
                style={{
                  color: serverInfo?.private_api
                    ? "var(--color-primary)"
                    : "var(--color-outline)",
                }}
              >
                {serverInfo?.private_api ? "Enabled" : "Disabled"}
              </span>
            }
          />
          <SettingsTile
            label="Proxy Service"
            trailing={
              <span style={{ color: "var(--color-on-surface-variant)" }}>
                {serverInfo?.proxy_service ?? "None"}
              </span>
            }
          />
          <SettingsTile
            label="Helper Connected"
            trailing={
              <span
                style={{
                  color: serverInfo?.helper_connected
                    ? "var(--color-primary)"
                    : "var(--color-outline)",
                }}
              >
                {serverInfo?.helper_connected ? "Yes" : "No"}
              </span>
            }
          />
        </SettingsSection>
      )}
    </>
  );
}

function DiagnosticsPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const { serverInfo, status } = useConnectionStore();
  const logLevel = settings["logLevel"] ?? "info";
  const enableFileLogging = settings["enableFileLogging"] !== "false";
  const maxLogFileSize = settings["maxLogFileSize"] ?? "10";
  const [logContents, setLogContents] = useState<string | null>(null);
  const [loadingLogs, setLoadingLogs] = useState(false);

  const handleViewLogs = useCallback(async () => {
    setLoadingLogs(true);
    try {
      // Read from Tauri log file via invoke if available, else show placeholder
      const { appLogDir } = await import("@tauri-apps/api/path");
      const logDir = await appLogDir();
      setLogContents(`Log directory: ${logDir}\n\nUse the "Open Log Folder" button to view log files.`);
    } catch {
      setLogContents("Logs are written to the app data directory.\nUse the button below to open the folder.");
    } finally {
      setLoadingLogs(false);
    }
  }, []);

  const handleOpenLogFolder = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      const { appLogDir } = await import("@tauri-apps/api/path");
      const logDir = await appLogDir();
      await open(logDir);
    } catch {
      // Fallback: try opening the general app data dir
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        const { appDataDir } = await import("@tauri-apps/api/path");
        const dataDir = await appDataDir();
        await open(dataDir);
      } catch {
        // silently fail
      }
    }
  }, []);

  return (
    <>
      <SettingsSection title="Connection Status">
        <SettingsTile
          label="Status"
          trailing={
            <span style={{
              color: status === "connected" ? "var(--color-primary)" :
                     status === "connecting" ? "var(--color-tertiary, orange)" :
                     "var(--color-error)",
              fontWeight: 600,
            }}>
              {status === "connected" ? "Connected" :
               status === "connecting" ? "Connecting..." :
               status === "error" ? "Error" : "Disconnected"}
            </span>
          }
        />
        {serverInfo && (
          <>
            <SettingsTile
              label="Server Version"
              trailing={<span style={{ color: "var(--color-on-surface-variant)" }}>{serverInfo.server_version ?? "Unknown"}</span>}
            />
            <SettingsTile
              label="Private API"
              trailing={
                <span style={{ color: serverInfo.private_api ? "var(--color-primary)" : "var(--color-error)" }}>
                  {serverInfo.private_api ? "Enabled" : "Disabled"}
                </span>
              }
            />
            <SettingsTile
              label="Helper Connected"
              trailing={
                <span style={{ color: serverInfo.helper_connected ? "var(--color-primary)" : "var(--color-error)" }}>
                  {serverInfo.helper_connected ? "Yes" : "No"}
                </span>
              }
            />
            <SettingsTile
              label="macOS Version"
              trailing={<span style={{ color: "var(--color-on-surface-variant)" }}>{serverInfo.os_version ?? "Unknown"}</span>}
            />
            <SettingsTile
              label="Proxy Service"
              trailing={<span style={{ color: "var(--color-on-surface-variant)" }}>{serverInfo.proxy_service ?? "None"}</span>}
            />
          </>
        )}
      </SettingsSection>

      <SettingsSection title="Logging">
        <SettingsDropdown
          label="Log Level"
          subtitle="Controls the verbosity of application logs"
          value={logLevel}
          options={[
            { label: "Error", value: "error" },
            { label: "Warning", value: "warn" },
            { label: "Info", value: "info" },
            { label: "Debug", value: "debug" },
            { label: "Trace", value: "trace" },
          ]}
          onChange={(v) => updateSetting("logLevel", v)}
        />
        <SettingsSwitch
          label="Enable File Logging"
          subtitle="Write logs to disk for troubleshooting"
          value={enableFileLogging}
          onChange={(v) => updateSetting("enableFileLogging", String(v))}
        />
        <SettingsDropdown
          label="Max Log File Size"
          subtitle="Maximum size before log rotation (MB)"
          value={maxLogFileSize}
          options={[
            { label: "5 MB", value: "5" },
            { label: "10 MB", value: "10" },
            { label: "25 MB", value: "25" },
            { label: "50 MB", value: "50" },
          ]}
          onChange={(v) => updateSetting("maxLogFileSize", v)}
        />
      </SettingsSection>

      <SettingsSection title="Log Viewer">
        <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
          <button
            onClick={handleViewLogs}
            disabled={loadingLogs}
            style={{
              padding: "6px 16px",
              borderRadius: 8,
              fontSize: "var(--font-body-small)",
              fontWeight: 600,
              backgroundColor: "var(--color-primary)",
              color: "var(--color-on-primary)",
              cursor: loadingLogs ? "not-allowed" : "pointer",
              opacity: loadingLogs ? 0.6 : 1,
            }}
          >
            {loadingLogs ? "Loading..." : "View Logs"}
          </button>
          <button
            onClick={handleOpenLogFolder}
            style={{
              padding: "6px 16px",
              borderRadius: 8,
              fontSize: "var(--font-body-small)",
              fontWeight: 600,
              backgroundColor: "transparent",
              color: "var(--color-primary)",
              border: "1px solid var(--color-outline)",
              cursor: "pointer",
            }}
          >
            Open Log Folder
          </button>
        </div>
        {logContents && (
          <pre
            style={{
              padding: 12,
              borderRadius: 8,
              backgroundColor: "var(--color-surface-variant)",
              color: "var(--color-on-surface)",
              fontSize: 12,
              fontFamily: "monospace",
              maxHeight: 300,
              overflow: "auto",
              whiteSpace: "pre-wrap",
              wordBreak: "break-all",
            }}
          >
            {logContents}
          </pre>
        )}
      </SettingsSection>

      <SettingsSection title="App Info">
        <SettingsTile
          label="App Data Location"
          subtitle="Local database, settings, and cache files"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)", fontSize: "var(--font-body-small)" }}>
              ~/bluebubbles
            </span>
          }
        />
        <SettingsTile
          label="Frontend"
          trailing={<span style={{ color: "var(--color-on-surface-variant)" }}>React + Vite</span>}
        />
        <SettingsTile
          label="Backend"
          trailing={<span style={{ color: "var(--color-on-surface-variant)" }}>Tauri 2 + Rust</span>}
        />
      </SettingsSection>
    </>
  );
}

function TroubleshootPanel() {
  const [syncing, setSyncing] = useState(false);
  const [syncingMessages, setSyncingMessages] = useState(false);
  const [syncResult, setSyncResult] = useState<string | null>(null);

  const handleResync = useCallback(async () => {
    setSyncing(true);
    setSyncResult(null);
    try {
      const result = await tauriSyncFull();
      setSyncResult(
        `Synced ${result.chats_synced} chats, ${result.handles_synced} handles, ${result.contacts_synced} contacts`
      );
    } catch (err: unknown) {
      setSyncResult(`Error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setSyncing(false);
    }
  }, []);

  const handleResyncMessages = useCallback(async () => {
    setSyncingMessages(true);
    setSyncResult(null);
    try {
      const result = await tauriSyncMessages(25);
      setSyncResult(`Synced ${result.messages_synced} messages from ${result.chats_synced} chats`);
    } catch (err: unknown) {
      setSyncResult(`Error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setSyncingMessages(false);
    }
  }, []);

  return (
    <>
      <SettingsSection title="Sync">
        <SettingsButton
          label="Re-sync Chats & Contacts"
          subtitle="Fetch latest chats, handles, and contacts from the server"
          buttonLabel={syncing ? "Syncing..." : "Sync Now"}
          onClick={handleResync}
          loading={syncing}
        />
        <SettingsButton
          label="Re-sync Messages"
          subtitle="Fetch recent messages for all chats (25 per chat)"
          buttonLabel={syncingMessages ? "Syncing..." : "Sync Messages"}
          onClick={handleResyncMessages}
          loading={syncingMessages}
        />
        {syncResult && (
          <div
            style={{
              padding: "8px 12px",
              borderRadius: 6,
              backgroundColor: syncResult.startsWith("Error")
                ? "var(--color-error-container)"
                : "var(--color-primary-container)",
              color: syncResult.startsWith("Error")
                ? "var(--color-error)"
                : "var(--color-on-primary-container)",
              fontSize: "var(--font-body-small)",
              marginTop: 4,
            }}
          >
            {syncResult}
          </div>
        )}
      </SettingsSection>

      <SettingsSection title="Diagnostics">
        <SettingsTile
          label="App Data Location"
          subtitle="Local database and configuration files"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)", fontSize: "var(--font-body-small)" }}>
              ~/bluebubbles
            </span>
          }
        />
      </SettingsSection>

      <SettingsSection title="Danger Zone">
        <div
          style={{
            padding: "12px 16px",
            borderRadius: 8,
            border: "1px solid var(--color-error)",
            backgroundColor: "var(--color-error-container)",
          }}
        >
          <div
            style={{
              fontSize: "var(--font-body-small)",
              color: "var(--color-on-error-container)",
              marginBottom: 8,
            }}
          >
            These actions cannot be undone. Proceed with caution.
          </div>
          <SettingsButton
            label="Clear Local Database"
            subtitle="Remove all locally cached messages and data"
            buttonLabel="Clear"
            onClick={() => {}}
            variant="error"
          />
        </div>
      </SettingsSection>
    </>
  );
}

function AboutPanel() {
  return (
    <>
      <SettingsSection title="About BlueBubbles">
        <SettingsTile
          label="Version"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)" }}>
              1.0.0-dev (Tauri)
            </span>
          }
        />
        <SettingsTile
          label="Platform"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)" }}>
              Desktop
            </span>
          }
        />
        <SettingsTile
          label="Engine"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)" }}>
              Tauri 2 + Rust
            </span>
          }
        />
      </SettingsSection>

      <SettingsSection title="Links">
        <SettingsTile
          label="GitHub Repository"
          subtitle="View the source code and contribute"
          onClick={() => window.open("https://github.com/BlueBubblesApp/bluebubbles-app", "_blank")}
          trailing={<span style={{ color: "var(--color-primary)", fontSize: 12 }}>{"\u2197"}</span>}
        />
        <SettingsTile
          label="Report an Issue"
          subtitle="Found a bug? Let us know"
          onClick={() => window.open("https://github.com/BlueBubblesApp/bluebubbles-app/issues", "_blank")}
          trailing={<span style={{ color: "var(--color-primary)", fontSize: 12 }}>{"\u2197"}</span>}
        />
        <SettingsTile
          label="Discord Community"
          subtitle="Join the BlueBubbles community"
          onClick={() => window.open("https://discord.gg/bluebubbles", "_blank")}
          trailing={<span style={{ color: "var(--color-primary)", fontSize: 12 }}>{"\u2197"}</span>}
        />
        <SettingsTile
          label="Make a Donation"
          subtitle="Support the development of BlueBubbles"
          onClick={() => window.open("https://bluebubbles.app/donate", "_blank")}
          trailing={<span style={{ color: "var(--color-primary)", fontSize: 12 }}>{"\u2197"}</span>}
        />
      </SettingsSection>
    </>
  );
}
