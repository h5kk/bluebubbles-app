/**
 * Loading spinner components.
 * Includes circular spinner and progress bar variants.
 */
import type { CSSProperties } from "react";

interface LoadingSpinnerProps {
  size?: number;
  color?: string;
}

export function LoadingSpinner({
  size = 24,
  color = "var(--color-primary)",
}: LoadingSpinnerProps) {
  const style: CSSProperties = {
    width: size,
    height: size,
    border: `3px solid transparent`,
    borderTopColor: color,
    borderRadius: "50%",
    animation: "spin 0.7s linear infinite",
    display: "inline-block",
  };

  return <div style={style} role="status" aria-label="Loading" />;
}

interface ProgressBarProps {
  progress: number; // 0-100
  height?: number;
}

export function ProgressBar({ progress, height = 4 }: ProgressBarProps) {
  const clamped = Math.max(0, Math.min(100, progress));

  return (
    <div
      style={{
        width: "100%",
        height,
        backgroundColor: "var(--color-surface-variant)",
        borderRadius: height / 2,
        overflow: "hidden",
      }}
      role="progressbar"
      aria-valuenow={clamped}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        style={{
          width: `${clamped}%`,
          height: "100%",
          backgroundColor: "var(--color-primary)",
          borderRadius: height / 2,
          transition: "width 300ms ease",
        }}
      />
    </div>
  );
}
