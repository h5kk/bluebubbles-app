/**
 * InputBar component - macOS Messages style.
 * Plus button (attachments), pill-shaped text field, audio/emoji/send buttons.
 */
import { useState, useCallback, useRef, type KeyboardEvent, type CSSProperties } from "react";

interface InputBarProps {
  onSend: (text: string) => void;
  onTyping?: (isTyping: boolean) => void;
  sending?: boolean;
  sendWithReturn?: boolean;
  placeholder?: string;
}

export function InputBar({
  onSend,
  onTyping,
  sending = false,
  sendWithReturn = false,
  placeholder = "iMessage",
}: InputBarProps) {
  const [text, setText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed || sending) return;
    onSend(trimmed);
    setText("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  }, [text, sending, onSend]);

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

  const hasText = text.trim().length > 0;
  const canSend = hasText && !sending;

  const containerStyle: CSSProperties = {
    display: "flex",
    alignItems: "flex-end",
    gap: 6,
    padding: "8px 12px",
    borderTop: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-background)",
    minHeight: "var(--input-bar-min-height)",
  };

  const inputWrapperStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    alignItems: "flex-end",
    backgroundColor: "var(--color-surface-variant)",
    borderRadius: 20,
    padding: "6px 12px",
    minHeight: 36,
    gap: 6,
  };

  return (
    <div style={containerStyle}>
      {/* Attachment / plus button */}
      <button
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
          placeholder={placeholder}
          rows={1}
          style={{
            flex: 1,
            resize: "none",
            maxHeight: 150,
            lineHeight: 1.4,
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface)",
            background: "transparent",
          }}
          aria-label="Message input"
        />

        {/* Right side buttons inside the pill */}
        <div style={{ display: "flex", alignItems: "center", gap: 4, flexShrink: 0, paddingBottom: 2 }}>
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
              <InputBarIconButton label="Emoji">
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
    </div>
  );
}

/* Small icon button used inside the input bar */
interface InputBarIconButtonProps {
  children: React.ReactNode;
  label: string;
  onClick?: () => void;
}

function InputBarIconButton({ children, label, onClick }: InputBarIconButtonProps) {
  return (
    <button
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
