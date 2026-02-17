/**
 * InputBar component - macOS Messages style.
 * Plus button (attachments), pill-shaped text field, audio/emoji/send buttons.
 */
import React, { useState, useCallback, useRef, type KeyboardEvent, type ClipboardEvent, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { EmojiPicker } from "./EmojiPicker";
import { useAttachmentStore } from "@/store/attachmentStore";
import { ImagePreviewCard } from "./ImagePreviewCard";

interface InputBarProps {
  onSend: (text: string) => void;
  onSendAttachment?: (file: File) => void;
  onTyping?: (isTyping: boolean) => void;
  sending?: boolean;
  sendWithReturn?: boolean;
  placeholder?: string;
  chatGuid?: string | null;
}

export function InputBar({
  onSend,
  onSendAttachment,
  onTyping,
  sending = false,
  sendWithReturn = false,
  placeholder = "iMessage",
  chatGuid = null,
}: InputBarProps) {
  const [text, setText] = useState("");
  const [pastedImage, setPastedImage] = useState<{ file: File; preview: string } | null>(null);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const emojiButtonRef = useRef<HTMLButtonElement>(null);

  const { pendingAttachments, addPendingAttachment, removePendingAttachment } =
    useAttachmentStore();

  // Filter attachments for current chat
  const currentChatAttachments = pendingAttachments.filter(
    (a) => a.chatGuid === chatGuid
  );

  const handleSend = useCallback(() => {
    if (pastedImage && onSendAttachment) {
      onSendAttachment(pastedImage.file);
      setPastedImage(null);
      return;
    }
    const trimmed = text.trim();
    if (!trimmed || sending) return;
    onSend(trimmed);
    setText("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  }, [text, sending, onSend, pastedImage, onSendAttachment]);

  const handlePaste = useCallback(
    async (e: ClipboardEvent<HTMLTextAreaElement>) => {
      const items = e.clipboardData?.items;
      if (!items) return;

      for (const item of Array.from(items)) {
        if (item.type.startsWith("image/")) {
          e.preventDefault();
          const file = item.getAsFile();
          if (file) {
            const preview = URL.createObjectURL(file);
            setPastedImage({ file, preview });
          }
          break;
        }
      }
    },
    []
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey && sendWithReturn) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend, sendWithReturn]
  );

  const handleInput = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const newValue = e.target.value;
      const wasEmpty = text.trim().length === 0;
      const isEmpty = newValue.trim().length === 0;
      setText(newValue);

      // Notify about typing state changes
      if (onTyping) {
        if (!isEmpty && wasEmpty) {
          onTyping(true);
        } else if (!isEmpty) {
          // Still typing - reset the timeout by re-signaling
          onTyping(true);
        } else if (isEmpty && !wasEmpty) {
          onTyping(false);
        }
      }

      const el = e.target;
      el.style.height = "auto";
      el.style.height = `${Math.min(el.scrollHeight, 150)}px`;
    },
    [text, onTyping]
  );

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file && chatGuid) {
        if (file.type.startsWith("image/")) {
          // Add to attachment store instead of local state
          addPendingAttachment(file, chatGuid);
        } else if (onSendAttachment) {
          onSendAttachment(file);
        }
      }
      // Reset input so same file can be selected again
      e.target.value = "";
    },
    [onSendAttachment, chatGuid, addPendingAttachment]
  );

  const handleAttachmentClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleEmojiSelect = useCallback(
    (emoji: string) => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newText = text.substring(0, start) + emoji + text.substring(end);

      setText(newText);

      // Set cursor position after emoji
      setTimeout(() => {
        textarea.focus();
        textarea.setSelectionRange(start + emoji.length, start + emoji.length);
      }, 0);
    },
    [text]
  );

  const hasText = text.trim().length > 0;
  const canSend = (hasText || pastedImage) && !sending;

  const containerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 6,
    padding: "8px 12px",
    borderTop: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-background)",
    minHeight: "var(--input-bar-min-height)",
    position: "relative",
  };

  const inputWrapperStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    alignItems: "center",
    backgroundColor: "var(--color-surface-variant)",
    borderRadius: 20,
    padding: "4px 12px",
    minHeight: 34,
    gap: 6,
  };

  return (
    <div style={containerStyle} data-input-bar>
      {/* Pending Attachments Preview */}
      <AnimatePresence>
        {currentChatAttachments.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 5 }}
            transition={{ duration: 0.2 }}
            style={{
              position: "absolute",
              bottom: "calc(100% + 8px)",
              left: 12,
              right: 12,
              display: "flex",
              flexDirection: "column",
              gap: 8,
              maxHeight: 300,
              overflowY: "auto",
            }}
          >
            {currentChatAttachments.map((attachment) => (
              <ImagePreviewCard
                key={attachment.id}
                attachment={attachment}
                onRemove={removePendingAttachment}
              />
            ))}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Pasted Image Preview (legacy) */}
      <AnimatePresence>
        {pastedImage && (
          <motion.div
            initial={{ opacity: 0, y: 10, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 5, scale: 0.95 }}
            transition={{ duration: 0.2 }}
            style={{
              position: "absolute",
              bottom: "calc(100% + 8px)",
              left: 12,
              backgroundColor: "var(--color-surface)",
              borderRadius: 12,
              padding: 8,
              boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
              border: "1px solid var(--color-surface-variant)",
              display: "flex",
              alignItems: "center",
              gap: 10,
            }}
          >
            <img
              src={pastedImage.preview}
              alt="Pasted screenshot"
              style={{
                width: 80,
                height: 80,
                borderRadius: 8,
                objectFit: "cover",
                border: "1px solid var(--color-outline)",
              }}
            />
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              <span style={{ fontSize: 12, fontWeight: 500, color: "var(--color-on-surface)" }}>
                Screenshot
              </span>
              <span style={{ fontSize: 11, color: "var(--color-on-surface-variant)" }}>
                Press Enter to send
              </span>
            </div>
            <button
              onClick={() => setPastedImage(null)}
              style={{
                width: 20,
                height: 20,
                borderRadius: "50%",
                backgroundColor: "var(--color-surface-variant)",
                color: "var(--color-on-surface-variant)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
                marginLeft: 8,
              }}
              aria-label="Remove screenshot"
            >
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.txt"
        onChange={handleFileSelect}
        style={{ display: "none" }}
      />

      {/* Attachment / plus button */}
      <button
        onClick={handleAttachmentClick}
        style={{
          width: 32,
          height: 32,
          borderRadius: "50%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "var(--color-surface-variant)",
          color: "var(--color-on-surface-variant)",
          fontSize: 18,
          fontWeight: 300,
          flexShrink: 0,
          cursor: "pointer",
          alignSelf: "center",
        }}
        aria-label="Add attachment"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M8 2V14M2 8H14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </svg>
      </button>

      {/* Text input area with right-side buttons */}
      <div style={inputWrapperStyle}>
        <textarea
          ref={textareaRef}
          value={text}
          onChange={handleInput}
          onKeyDown={handleKeyDown}
          onPaste={handlePaste}
          placeholder={placeholder}
          rows={1}
          style={{
            flex: 1,
            resize: "none",
            maxHeight: 150,
            lineHeight: "20px",
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface)",
            background: "transparent",
            padding: "4px 0",
            margin: 0,
          }}
          aria-label="Message input"
        />

        {/* Right side buttons inside the pill */}
        <div style={{ display: "flex", alignItems: "center", gap: 4, flexShrink: 0 }}>
          {!hasText && (
            <>
              {/* Audio / waveform icon */}
              <InputBarIconButton label="Audio message">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <rect x="6" y="1" width="4" height="9" rx="2" stroke="currentColor" strokeWidth="1.3" fill="none" />
                  <path d="M3 7C3 10.3 5.2 12 8 12C10.8 12 13 10.3 13 7" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" fill="none" />
                  <path d="M8 12V15M6 15H10" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
                </svg>
              </InputBarIconButton>

              {/* Emoji / smiley face icon */}
              <InputBarIconButton
                ref={emojiButtonRef}
                label="Emoji"
                onClick={() => setShowEmojiPicker((prev) => !prev)}
              >
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <circle cx="8" cy="8" r="7" stroke="currentColor" strokeWidth="1.3" fill="none" />
                  <circle cx="5.5" cy="6.5" r="1" fill="currentColor" />
                  <circle cx="10.5" cy="6.5" r="1" fill="currentColor" />
                  <path d="M5 10C5.8 11.3 7 12 8 12C9 12 10.2 11.3 11 10" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" fill="none" />
                </svg>
              </InputBarIconButton>
            </>
          )}

          {/* Send button - appears when there is text */}
          {hasText && (
            <button
              onClick={handleSend}
              disabled={!canSend}
              style={{
                width: 24,
                height: 24,
                borderRadius: "50%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                backgroundColor: canSend ? "#007AFF" : "var(--color-outline)",
                color: "#FFFFFF",
                transition: "background-color 150ms ease",
                flexShrink: 0,
                cursor: canSend ? "pointer" : "default",
              }}
              aria-label="Send message"
            >
              {sending ? (
                <span
                  style={{
                    width: 12,
                    height: 12,
                    border: "1.5px solid currentColor",
                    borderTopColor: "transparent",
                    borderRadius: "50%",
                    animation: "spin 0.6s linear infinite",
                  }}
                />
              ) : (
                /* Upward arrow */
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                  <path d="M6 10V2M6 2L2.5 5.5M6 2L9.5 5.5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              )}
            </button>
          )}
        </div>
      </div>

      {/* Emoji picker */}
      <AnimatePresence>
        {showEmojiPicker && (
          <EmojiPicker
            onEmojiSelect={handleEmojiSelect}
            onClose={() => setShowEmojiPicker(false)}
            anchorRef={emojiButtonRef}
          />
        )}
      </AnimatePresence>
    </div>
  );
}

/* Small icon button used inside the input bar */
interface InputBarIconButtonProps {
  children: React.ReactNode;
  label: string;
  onClick?: () => void;
}

const InputBarIconButton = React.forwardRef<HTMLButtonElement, InputBarIconButtonProps>(
  ({ children, label, onClick }, ref) => {
    return (
      <button
        ref={ref}
        onClick={onClick}
        aria-label={label}
        style={{
          width: 24,
          height: 24,
          borderRadius: "50%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--color-on-surface-variant)",
          background: "transparent",
          cursor: "pointer",
          flexShrink: 0,
        }}
      >
        {children}
      </button>
    );
  }
);
