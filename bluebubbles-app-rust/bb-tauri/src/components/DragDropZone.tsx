/**
 * DragDropZone - wrapper that handles file drag and drop for conversations.
 * Shows an overlay when files are being dragged over the drop zone.
 */
import { useState, useCallback, type DragEvent, type CSSProperties, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface DragDropZoneProps {
  children: ReactNode;
  onFileDrop: (files: File[]) => void;
  disabled?: boolean;
  accept?: string;
}

export function DragDropZone({
  children,
  onFileDrop,
  disabled = false,
  accept = "image/*",
}: DragDropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [dragDepth, setDragDepth] = useState(0);

  const handleDragEnter = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (disabled) return;

      setDragDepth((prev) => prev + 1);

      if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
        // Check if any of the dragged items are files
        const hasFiles = Array.from(e.dataTransfer.items).some(
          (item) => item.kind === "file"
        );
        if (hasFiles) {
          setIsDragging(true);
        }
      }
    },
    [disabled]
  );

  const handleDragLeave = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (disabled) return;

      setDragDepth((prev) => {
        const newDepth = prev - 1;
        if (newDepth <= 0) {
          setIsDragging(false);
          return 0;
        }
        return newDepth;
      });
    },
    [disabled]
  );

  const handleDragOver = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      if (disabled) return;

      // Set dropEffect to show copy cursor
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = "copy";
      }
    },
    [disabled]
  );

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      e.stopPropagation();

      setIsDragging(false);
      setDragDepth(0);

      if (disabled) return;

      const { files } = e.dataTransfer;
      if (files && files.length > 0) {
        const fileArray = Array.from(files);

        // Filter files by accept type if specified
        const acceptedFiles = fileArray.filter((file) => {
          if (!accept || accept === "*") return true;

          const acceptTypes = accept.split(",").map((t) => t.trim());
          return acceptTypes.some((type) => {
            if (type.endsWith("/*")) {
              const category = type.split("/")[0];
              return file.type.startsWith(`${category}/`);
            }
            return file.type === type;
          });
        });

        if (acceptedFiles.length > 0) {
          onFileDrop(acceptedFiles);
        }
      }
    },
    [disabled, accept, onFileDrop]
  );

  const containerStyle: CSSProperties = {
    position: "relative",
    width: "100%",
    height: "100%",
    display: "flex",
    flexDirection: "column",
    minHeight: 0,
    minWidth: 0,
  };

  const overlayStyle: CSSProperties = {
    position: "absolute",
    inset: 0,
    backgroundColor: "rgba(0, 122, 255, 0.1)",
    backdropFilter: "blur(4px)",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: 12,
    zIndex: 1000,
    border: "3px dashed #007AFF",
    borderRadius: 12,
    pointerEvents: "none",
  };

  const iconStyle: CSSProperties = {
    width: 64,
    height: 64,
    borderRadius: "50%",
    backgroundColor: "#007AFF",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    color: "#FFFFFF",
  };

  const textStyle: CSSProperties = {
    fontSize: "var(--font-body-large)",
    fontWeight: 600,
    color: "#007AFF",
    textAlign: "center",
  };

  const subtextStyle: CSSProperties = {
    fontSize: "var(--font-body-medium)",
    color: "var(--color-on-surface-variant)",
    textAlign: "center",
  };

  return (
    <div
      style={containerStyle}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {children}

      <AnimatePresence>
        {isDragging && !disabled && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            style={overlayStyle}
          >
            <motion.div
              initial={{ scale: 0.8 }}
              animate={{ scale: 1 }}
              transition={{ duration: 0.2, ease: "backOut" }}
              style={iconStyle}
            >
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
                <path
                  d="M12 4V20M12 4L8 8M12 4L16 8"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <path
                  d="M4 17V19C4 20.1 4.9 21 6 21H18C19.1 21 20 20.1 20 19V17"
                  stroke="currentColor"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </motion.div>
            <div style={textStyle}>Drop to attach</div>
            <div style={subtextStyle}>Release to add files to this conversation</div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
