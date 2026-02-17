/**
 * ImageLightbox - full-screen image viewer with zoom, pan, and navigation.
 * Supports keyboard navigation (ESC to close, arrow keys to navigate).
 */
import { useEffect, useCallback, useState, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useAttachmentStore } from "@/store/attachmentStore";

export function ImageLightbox() {
  const { lightbox, closeLightbox, nextImage, previousImage, setCurrentIndex } =
    useAttachmentStore();
  const { isOpen, currentIndex, images } = lightbox;

  const [isZoomed, setIsZoomed] = useState(false);
  const [scale, setScale] = useState(1);

  const currentImage = images[currentIndex];
  const hasMultiple = images.length > 1;
  const hasPrevious = currentIndex > 0;
  const hasNext = currentIndex < images.length - 1;

  // Reset zoom when image changes
  useEffect(() => {
    setIsZoomed(false);
    setScale(1);
  }, [currentIndex]);

  // Keyboard navigation
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        closeLightbox();
      } else if (e.key === "ArrowLeft" && hasPrevious) {
        previousImage();
      } else if (e.key === "ArrowRight" && hasNext) {
        nextImage();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, hasPrevious, hasNext, closeLightbox, previousImage, nextImage]);

  const handleImageClick = useCallback(() => {
    if (isZoomed) {
      setIsZoomed(false);
      setScale(1);
    } else {
      setIsZoomed(true);
      setScale(2);
    }
  }, [isZoomed]);

  const handleDownload = useCallback(() => {
    if (!currentImage) return;

    const link = document.createElement("a");
    link.href = currentImage.src;
    link.download = currentImage.alt || "image";
    link.click();
  }, [currentImage]);

  if (!isOpen || !currentImage) return null;

  const overlayStyle: CSSProperties = {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.95)",
    zIndex: 9999,
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
  };

  const headerStyle: CSSProperties = {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    height: 60,
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0 20px",
    background: "linear-gradient(to bottom, rgba(0,0,0,0.5), transparent)",
    zIndex: 2,
  };

  const titleStyle: CSSProperties = {
    color: "#FFFFFF",
    fontSize: "var(--font-body-large)",
    fontWeight: 500,
  };

  const buttonGroupStyle: CSSProperties = {
    display: "flex",
    gap: 12,
  };

  const iconButtonStyle: CSSProperties = {
    width: 40,
    height: 40,
    borderRadius: "50%",
    backgroundColor: "rgba(255, 255, 255, 0.15)",
    color: "#FFFFFF",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    cursor: "pointer",
    transition: "background-color 150ms ease",
    border: "none",
  };

  const imageContainerStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    width: "100%",
    position: "relative",
    overflow: isZoomed ? "auto" : "hidden",
    cursor: isZoomed ? "zoom-out" : "zoom-in",
  };

  const imageStyle: CSSProperties = {
    maxWidth: isZoomed ? "none" : "90%",
    maxHeight: isZoomed ? "none" : "90%",
    objectFit: "contain",
    transform: `scale(${scale})`,
    transition: "transform 300ms cubic-bezier(0.4, 0, 0.2, 1)",
  };

  const navButtonStyle: CSSProperties = {
    position: "absolute",
    top: "50%",
    transform: "translateY(-50%)",
    width: 48,
    height: 48,
    borderRadius: "50%",
    backgroundColor: "rgba(255, 255, 255, 0.2)",
    color: "#FFFFFF",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    cursor: "pointer",
    transition: "background-color 150ms ease",
    border: "none",
    zIndex: 2,
  };

  const counterStyle: CSSProperties = {
    position: "absolute",
    bottom: 20,
    left: "50%",
    transform: "translateX(-50%)",
    backgroundColor: "rgba(0, 0, 0, 0.7)",
    color: "#FFFFFF",
    padding: "8px 16px",
    borderRadius: 20,
    fontSize: "var(--font-body-medium)",
    zIndex: 2,
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.2 }}
        style={overlayStyle}
        onClick={(e) => {
          if (e.target === e.currentTarget) closeLightbox();
        }}
      >
        {/* Header with title and controls */}
        <div style={headerStyle}>
          <div style={titleStyle}>{currentImage.alt || "Image"}</div>
          <div style={buttonGroupStyle}>
            <button
              onClick={handleDownload}
              style={iconButtonStyle}
              aria-label="Download image"
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor =
                  "rgba(255, 255, 255, 0.3)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor =
                  "rgba(255, 255, 255, 0.15)";
              }}
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path
                  d="M10 2V14M10 14L6 10M10 14L14 10"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <path
                  d="M3 17H17"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
              </svg>
            </button>
            <button
              onClick={closeLightbox}
              style={iconButtonStyle}
              aria-label="Close lightbox"
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor =
                  "rgba(255, 255, 255, 0.3)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor =
                  "rgba(255, 255, 255, 0.15)";
              }}
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path
                  d="M4 4L16 16M16 4L4 16"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
              </svg>
            </button>
          </div>
        </div>

        {/* Main image */}
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          transition={{ duration: 0.2 }}
          style={imageContainerStyle}
          onClick={handleImageClick}
        >
          <img
            src={currentImage.src}
            alt={currentImage.alt || "Image"}
            style={imageStyle}
          />
        </motion.div>

        {/* Navigation arrows */}
        {hasMultiple && hasPrevious && (
          <button
            onClick={previousImage}
            style={{ ...navButtonStyle, left: 20 }}
            aria-label="Previous image"
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.4)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.2)";
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <path
                d="M15 18L9 12L15 6"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}

        {hasMultiple && hasNext && (
          <button
            onClick={nextImage}
            style={{ ...navButtonStyle, right: 20 }}
            aria-label="Next image"
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.4)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.2)";
            }}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <path
                d="M9 18L15 12L9 6"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}

        {/* Image counter */}
        {hasMultiple && (
          <div style={counterStyle}>
            {currentIndex + 1} / {images.length}
          </div>
        )}

        {/* Thumbnail strip for multiple images */}
        {hasMultiple && images.length <= 10 && (
          <div
            style={{
              position: "absolute",
              bottom: 60,
              left: "50%",
              transform: "translateX(-50%)",
              display: "flex",
              gap: 8,
              padding: "12px 16px",
              backgroundColor: "rgba(0, 0, 0, 0.7)",
              borderRadius: 12,
              zIndex: 2,
            }}
          >
            {images.map((img, idx) => (
              <button
                key={idx}
                onClick={() => setCurrentIndex(idx)}
                style={{
                  width: 48,
                  height: 48,
                  borderRadius: 6,
                  overflow: "hidden",
                  border:
                    idx === currentIndex
                      ? "2px solid #007AFF"
                      : "2px solid transparent",
                  cursor: "pointer",
                  opacity: idx === currentIndex ? 1 : 0.6,
                  transition: "opacity 150ms ease",
                  padding: 0,
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.opacity = "1";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.opacity =
                    idx === currentIndex ? "1" : "0.6";
                }}
              >
                <img
                  src={img.src}
                  alt={`Thumbnail ${idx + 1}`}
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "cover",
                  }}
                />
              </button>
            ))}
          </div>
        )}
      </motion.div>
    </AnimatePresence>
  );
}
