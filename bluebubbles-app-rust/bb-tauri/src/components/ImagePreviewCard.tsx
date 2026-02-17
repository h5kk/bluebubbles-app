/**
 * ImagePreviewCard - shows a preview of an image being attached to a message.
 * Displays thumbnail, filename, size, remove button, and upload progress.
 */
import { memo, type CSSProperties } from "react";
import { motion } from "framer-motion";
import type { PendingAttachment } from "@/store/attachmentStore";

interface ImagePreviewCardProps {
  attachment: PendingAttachment;
  onRemove: (id: string) => void;
}

export const ImagePreviewCard = memo(function ImagePreviewCard({
  attachment,
  onRemove,
}: ImagePreviewCardProps) {
  const { id, file, preview, progress, error } = attachment;

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const containerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    backgroundColor: "var(--color-surface)",
    borderRadius: 12,
    padding: 8,
    border: error
      ? "1px solid var(--color-error)"
      : "1px solid var(--color-surface-variant)",
    boxShadow: "0 2px 8px rgba(0,0,0,0.1)",
    position: "relative",
    overflow: "hidden",
  };

  const thumbnailStyle: CSSProperties = {
    width: 64,
    height: 64,
    borderRadius: 8,
    objectFit: "cover",
    flexShrink: 0,
    border: "1px solid var(--color-outline)",
  };

  const infoStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    gap: 4,
    minWidth: 0,
  };

  const filenameStyle: CSSProperties = {
    fontSize: "var(--font-body-medium)",
    fontWeight: 500,
    color: "var(--color-on-surface)",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  };

  const metaStyle: CSSProperties = {
    fontSize: "var(--font-label-small)",
    color: "var(--color-on-surface-variant)",
  };

  const errorStyle: CSSProperties = {
    fontSize: "var(--font-label-small)",
    color: "var(--color-error)",
    fontWeight: 500,
  };

  const removeButtonStyle: CSSProperties = {
    width: 28,
    height: 28,
    borderRadius: "50%",
    backgroundColor: "var(--color-surface-variant)",
    color: "var(--color-on-surface-variant)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    cursor: "pointer",
    flexShrink: 0,
    transition: "background-color 150ms ease",
  };

  const progressBarStyle: CSSProperties = {
    position: "absolute",
    bottom: 0,
    left: 0,
    right: 0,
    height: 3,
    backgroundColor: "var(--color-surface-variant)",
    overflow: "hidden",
  };

  const progressFillStyle: CSSProperties = {
    height: "100%",
    backgroundColor: "#007AFF",
    transition: "width 200ms ease",
    width: `${progress}%`,
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: -10, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: -5, scale: 0.95 }}
      transition={{ duration: 0.2 }}
      style={containerStyle}
    >
      <img src={preview} alt="Preview" style={thumbnailStyle} />

      <div style={infoStyle}>
        <div style={filenameStyle}>{file.name}</div>
        <div style={metaStyle}>{formatFileSize(file.size)}</div>
        {error && <div style={errorStyle}>{error}</div>}
        {progress > 0 && progress < 100 && !error && (
          <div style={metaStyle}>Uploading {progress}%</div>
        )}
      </div>

      <button
        onClick={() => onRemove(id)}
        style={removeButtonStyle}
        aria-label="Remove attachment"
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor =
            "var(--color-error-container)";
          e.currentTarget.style.color = "var(--color-error)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor =
            "var(--color-surface-variant)";
          e.currentTarget.style.color = "var(--color-on-surface-variant)";
        }}
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M1 1L11 11M11 1L1 11"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </svg>
      </button>

      {progress > 0 && progress < 100 && !error && (
        <div style={progressBarStyle}>
          <div style={progressFillStyle} />
        </div>
      )}
    </motion.div>
  );
});
