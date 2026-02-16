/**
 * Custom title bar component.
 * Draggable region with window controls (minimize, maximize, close).
 * Implements the TitleBar from spec 07-shared-components.md section 2.
 */
import { useState, useCallback, type CSSProperties } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface TitleBarProps {
  title?: string;
}

export function TitleBar({ title = "BlueBubbles" }: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false);

  const handleMinimize = useCallback(async () => {
    const appWindow = getCurrentWindow();
    await appWindow.minimize();
  }, []);

  const handleMaximize = useCallback(async () => {
    const appWindow = getCurrentWindow();
    const maximized = await appWindow.isMaximized();
    if (maximized) {
      await appWindow.unmaximize();
      setIsMaximized(false);
    } else {
      await appWindow.maximize();
      setIsMaximized(true);
    }
  }, []);

  const handleClose = useCallback(async () => {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  }, []);

  const barStyle: CSSProperties = {
    height: "var(--title-bar-height)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    backgroundColor: "var(--color-surface)",
    borderBottom: "1px solid var(--color-surface-variant)",
    userSelect: "none",
    WebkitUserSelect: "none",
    position: "relative",
    zIndex: 100,
  };

  const dragRegionStyle: CSSProperties = {
    flex: 1,
    height: "100%",
    display: "flex",
    alignItems: "center",
    paddingLeft: 16,
    // @ts-expect-error Tauri-specific CSS property for drag region
    WebkitAppRegion: "drag",
  };

  const controlsStyle: CSSProperties = {
    display: "flex",
    height: "100%",
    // @ts-expect-error Tauri-specific CSS property
    WebkitAppRegion: "no-drag",
  };

  const buttonBase: CSSProperties = {
    width: 46,
    height: "100%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: 12,
    color: "var(--color-on-surface)",
    transition: "background-color 100ms ease",
    cursor: "default",
  };

  return (
    <div style={barStyle} data-tauri-drag-region>
      <div style={dragRegionStyle} data-tauri-drag-region>
        <span
          style={{
            fontSize: "var(--font-body-medium)",
            fontWeight: 500,
            color: "var(--color-on-surface)",
            letterSpacing: "0.3px",
          }}
        >
          {title}
        </span>
      </div>

      <div style={controlsStyle}>
        <WindowButton
          style={buttonBase}
          onClick={handleMinimize}
          hoverBg="var(--color-surface-variant)"
          label="Minimize"
        >
          <svg width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </WindowButton>

        <WindowButton
          style={buttonBase}
          onClick={handleMaximize}
          hoverBg="var(--color-surface-variant)"
          label={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path
                d="M2 0h6v2h2v6H8v2H0V4h2V0zm1 1v3h5V1H3zm-2 4v4h6V5H1z"
                fill="currentColor"
                fillRule="evenodd"
              />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10">
              <rect
                width="9"
                height="9"
                x="0.5"
                y="0.5"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
              />
            </svg>
          )}
        </WindowButton>

        <WindowButton
          style={buttonBase}
          onClick={handleClose}
          hoverBg="#E81123"
          hoverColor="#FFFFFF"
          label="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path
              d="M0 0L10 10M10 0L0 10"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </WindowButton>
      </div>
    </div>
  );
}

interface WindowButtonProps {
  children: React.ReactNode;
  style: CSSProperties;
  onClick: () => void;
  hoverBg: string;
  hoverColor?: string;
  label: string;
}

function WindowButton({
  children,
  style,
  onClick,
  hoverBg,
  hoverColor,
  label,
}: WindowButtonProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      style={{
        ...style,
        backgroundColor: hovered ? hoverBg : "transparent",
        color: hovered && hoverColor ? hoverColor : style.color,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      aria-label={label}
    >
      {children}
    </button>
  );
}
