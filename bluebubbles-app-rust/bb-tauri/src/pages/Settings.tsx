/**
 * Settings page - application settings hub.
 * Implements spec 04-settings-screens.md.
 */
import { useState, useCallback, type CSSProperties, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSettingsStore } from "@/store/settingsStore";
import { useConnectionStore } from "@/store/connectionStore";
import { useTheme, type ThemeMode } from "@/hooks/useTheme";

type SettingsPanel =
  | "general"
  | "appearance"
  | "chat"
  | "notifications"
  | "server"
  | "about";

export function Settings() {
  const [activePanel, setActivePanel] = useState<SettingsPanel>("general");

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
            label="Server"
            icon={"\uD83D\uDDA5\uFE0F"}
            active={activePanel === "server"}
            onClick={() => setActivePanel("server")}
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
              {activePanel === "server" && <ServerPanel />}
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
}: {
  label: string;
  subtitle?: string;
  trailing?: ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "10px 0",
        gap: 16,
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
}: {
  label: string;
  subtitle?: string;
  value: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <SettingsTile
      label={label}
      subtitle={subtitle}
      trailing={
        <button
          onClick={() => onChange(!value)}
          style={{
            width: 44,
            height: 24,
            borderRadius: 12,
            backgroundColor: value ? "var(--color-primary)" : "var(--color-outline)",
            position: "relative",
            transition: "background-color 200ms ease",
            cursor: "pointer",
          }}
          role="switch"
          aria-checked={value}
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

// ─── Panels ───────────────────────────────────────────────────────────────────

function GeneralPanel() {
  const { sendWithReturn, tabletMode, updateSetting } = useSettingsStore();

  return (
    <>
      <SettingsSection title="Input">
        <SettingsSwitch
          label="Send with Return"
          subtitle="Press Enter to send messages instead of adding a new line"
          value={sendWithReturn}
          onChange={(v) => updateSetting("sendWithReturn", String(v))}
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
  } = useSettingsStore();

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
            { label: "Material", value: "material" },
            { label: "iOS", value: "ios" },
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
      </SettingsSection>
    </>
  );
}

function ChatPanel() {
  const { updateSetting, settings } = useSettingsStore();
  const showDeliveryReceipts = settings["showDeliveryReceipts"] !== "false";
  const showTypingIndicators = settings["showTypingIndicators"] !== "false";
  const autoSaveMedia = settings["autoSaveMedia"] === "true";

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
      </SettingsSection>

      <SettingsSection title="Media">
        <SettingsSwitch
          label="Auto-Save Media"
          subtitle="Automatically download images and videos"
          value={autoSaveMedia}
          onChange={(v) => updateSetting("autoSaveMedia", String(v))}
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

  return (
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
    </SettingsSection>
  );
}

function ServerPanel() {
  const { serverInfo } = useConnectionStore();

  return (
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
    </SettingsSection>
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
              1.0.0-dev
            </span>
          }
        />
        <SettingsTile
          label="Platform"
          trailing={
            <span style={{ color: "var(--color-on-surface-variant)" }}>
              Tauri Desktop
            </span>
          }
        />
      </SettingsSection>

      <SettingsSection title="Links">
        <SettingsTile
          label="GitHub Repository"
          subtitle="View the source code and contribute"
        />
        <SettingsTile
          label="Report an Issue"
          subtitle="Found a bug? Let us know"
        />
        <SettingsTile
          label="Discord Community"
          subtitle="Join the BlueBubbles community"
        />
      </SettingsSection>
    </>
  );
}
