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
    backgroundColor: "var(--color-search-bg)",
    borderRadius: 18,
    padding: "6px 14px",
    margin: 0,
  };

  return (
    <div style={containerStyle}>
      <svg
        width="15"
        height="15"
        viewBox="0 0 15 15"
        fill="none"
        style={{ flexShrink: 0 }}
      >
        <circle
          cx="6.5"
          cy="6.5"
          r="5"
          stroke="var(--color-search-icon)"
          strokeWidth="1.5"
          fill="none"
        />
        <line
          x1="10.5"
          y1="10.5"
          x2="14"
          y2="14"
          stroke="var(--color-search-icon)"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
      <input
        type="text"
        value={localValue}
        onChange={handleChange}
        placeholder={placeholder}
        autoFocus={autoFocus}
        className="search-input"
        style={{
          flex: 1,
          fontSize: "var(--font-body-medium)",
          color: "var(--color-search-on)",
          pointerEvents: "auto",
          userSelect: "text",
          WebkitUserSelect: "text",
          caretColor: "var(--color-search-on)",
        }}
        aria-label={placeholder}
      />
      {localValue && (
        <button
          onClick={handleClear}
          style={{
            color: "var(--color-search-icon)",
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
