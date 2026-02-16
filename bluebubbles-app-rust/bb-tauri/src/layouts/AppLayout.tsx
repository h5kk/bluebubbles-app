/**
 * Main application layout component.
 * Split view with sidebar (conversation list) and content area.
 * Implements the layout structure from spec 00-ui-overview.md.
 */
import { type CSSProperties } from "react";
import { Outlet } from "react-router-dom";
import { TitleBar } from "./TitleBar";
import { Sidebar } from "./Sidebar";
import { ConversationList } from "@/pages/ConversationList";

export function AppLayout() {
  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100vh",
    width: "100vw",
    overflow: "hidden",
    backgroundColor: "var(--color-background)",
  };

  const bodyStyle: CSSProperties = {
    display: "flex",
    flex: 1,
    overflow: "hidden",
  };

  const contentStyle: CSSProperties = {
    flex: 1,
    overflow: "hidden",
    display: "flex",
    flexDirection: "column",
    backgroundColor: "var(--color-background)",
  };

  return (
    <div style={containerStyle}>
      <TitleBar />
      <div style={bodyStyle}>
        <Sidebar>
          <ConversationList />
        </Sidebar>
        <div style={contentStyle}>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
