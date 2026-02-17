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
  const sidebarWidth = 315;
  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100vh",
    width: "100vw",
    overflow: "hidden",
    backgroundColor: "var(--color-background)",
    position: "relative",
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
    padding: 0,
    paddingTop: "calc(var(--title-bar-height) - 10px)",
    boxSizing: "border-box",
    minHeight: 0, // Allow flex items to shrink below content size
  };

  return (
    <div style={containerStyle}>
      <TitleBar title="" overlay offsetLeft={sidebarWidth} />
      <div style={bodyStyle}>
        <Sidebar width={sidebarWidth}>
          <ConversationList />
        </Sidebar>
        <div style={contentStyle} data-app-content>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
