/**
 * LoadingLine - iOS-style thin blue progress line.
 * Animates left-to-right continuously while visible.
 * Reusable across sidebar header, conversation view, etc.
 */
import { motion, AnimatePresence } from "framer-motion";
import type { CSSProperties } from "react";

interface LoadingLineProps {
  visible: boolean;
  color?: string;
  height?: number;
  style?: CSSProperties;
}

export function LoadingLine({
  visible,
  color = "#007AFF",
  height = 2,
  style,
}: LoadingLineProps) {
  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          style={{
            position: "relative",
            width: "100%",
            height,
            overflow: "hidden",
            backgroundColor: `${color}20`,
            flexShrink: 0,
            ...style,
          }}
        >
          <motion.div
            initial={{ x: "-100%" }}
            animate={{ x: "100%" }}
            transition={{
              repeat: Infinity,
              duration: 1.2,
              ease: "easeInOut",
            }}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "40%",
              height: "100%",
              background: `linear-gradient(90deg, transparent, ${color}, transparent)`,
              borderRadius: height / 2,
            }}
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}
