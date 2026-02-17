/**
 * SearchBar component with debounce support.
 */
import { useState, useCallback, useRef, useEffect, type CSSProperties } from "react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  debounceMs?: number;
  autoFocus?: boolean;
}

export function SearchBar({
  value,
  onChange,
  placeholder = "Search",
  debounceMs = 250,
  autoFocus = false,
}: SearchBarProps) {
  const [localValue, setLocalValue] = useState(value);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      setLocalValue(newValue);

      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        onChange(newValue);
      }, debounceMs);
    },
    [onChange, debounceMs]
  );

  const handleClear = useCallback(() => {
    setLocalValue("");
    onChange("");
  }, [onChange]);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const containerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 8,
    backgroundColor: "var(--color-surface-variant)",
    borderRadius: 18,
    padding: "6px 14px",
    margin: 0,
  };

  return (
    <div style={containerStyle}>
      <span
        style={{
          color: "var(--color-outline)",
          fontSize: 14,
          flexShrink: 0,
        }}
      >
        {"\uD83D\uDD0D"}
      </span>
      <input
        type="text"
        value={localValue}
        onChange={handleChange}
        placeholder={placeholder}
        autoFocus={autoFocus}
        style={{
          flex: 1,
          fontSize: "var(--font-body-medium)",
          color: "var(--color-on-surface)",
        }}
        aria-label={placeholder}
      />
      {localValue && (
        <button
          onClick={handleClear}
          style={{
            color: "var(--color-outline)",
            fontSize: 14,
            flexShrink: 0,
            padding: "2px 4px",
          }}
          aria-label="Clear search"
        >
          {"\u2715"}
        </button>
      )}
    </div>
  );
}
