/**
 * InputBar component for the conversation text field.
 * Implements the input bar from spec 03-conversation-view.md section 11.
 */
import { useState, useCallback, useRef, type KeyboardEvent, type CSSProperties } from "react";

interface InputBarProps {
  onSend: (text: string) => void;
  sending?: boolean;
  sendWithReturn?: boolean;
  placeholder?: string;
}

export function InputBar({
  onSend,
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
      setText(e.target.value);
      // Auto-resize
      const el = e.target;
      el.style.height = "auto";
      el.style.height = `${Math.min(el.scrollHeight, 150)}px`;
    },
    []
  );

  const containerStyle: CSSProperties = {
    display: "flex",
    alignItems: "flex-end",
    gap: 8,
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
    borderRadius: 22,
    padding: "6px 16px",
    minHeight: 36,
  };

  const canSend = text.trim().length > 0 && !sending;

  return (
    <div style={containerStyle}>
      {/* Attachment button */}
      <button
        style={{
          width: 36,
          height: 36,
          borderRadius: "50%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--color-primary)",
          fontSize: 20,
          flexShrink: 0,
        }}
        aria-label="Add attachment"
      >
        +
      </button>

      {/* Text input */}
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
      </div>

      {/* Send button */}
      <button
        onClick={handleSend}
        disabled={!canSend}
        style={{
          width: 36,
          height: 36,
          borderRadius: "50%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: canSend
            ? "var(--color-primary)"
            : "var(--color-outline)",
          color: canSend
            ? "var(--color-on-primary)"
            : "var(--color-on-surface-variant)",
          transition: "background-color 150ms ease, color 150ms ease",
          flexShrink: 0,
          fontSize: 16,
        }}
        aria-label="Send message"
      >
        {sending ? (
          <span
            style={{
              width: 16,
              height: 16,
              border: "2px solid currentColor",
              borderTopColor: "transparent",
              borderRadius: "50%",
              animation: "spin 0.6s linear infinite",
            }}
          />
        ) : (
          "\u2191"
        )}
      </button>
    </div>
  );
}
