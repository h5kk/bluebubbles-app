/**
 * OTP Toast Demo Page
 *
 * Test page for demonstrating OTP toast functionality.
 * Allows manual triggering of OTP notifications for testing.
 */
import { useState, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { useOtpToast } from "@/contexts/OtpToastContext";

export function OtpDemo() {
  const navigate = useNavigate();
  const { showOtp } = useOtpToast();
  const [customCode, setCustomCode] = useState("123456");
  const [customSnippet, setCustomSnippet] = useState(
    "Your verification code is 123456. Do not share this code with anyone."
  );

  const presetExamples = [
    {
      code: "123456",
      snippet: "Your verification code is 123456. Do not share this code with anyone.",
      label: "Standard 6-digit",
    },
    {
      code: "8472",
      snippet: "Use code 8472 to verify your Apple ID. This code will expire in 10 minutes.",
      label: "Apple ID (4-digit)",
    },
    {
      code: "G8K3P2",
      snippet: "Your Google verification code is G8K3P2. Valid for 5 minutes.",
      label: "Alphanumeric",
    },
    {
      code: "759283",
      snippet: "Amazon: 759283 is your one-time password. Do not share it with anyone.",
      label: "Amazon OTP",
    },
    {
      code: "942156",
      snippet: "[WhatsApp] Your code is 942156. Don't share it with others.",
      label: "WhatsApp",
    },
  ];

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    padding: "16px 24px",
    borderBottom: "1px solid var(--color-surface-variant)",
    flexShrink: 0,
  };

  const contentStyle: CSSProperties = {
    flex: 1,
    overflow: "auto",
    padding: 24,
  };

  const sectionStyle: CSSProperties = {
    marginBottom: 32,
  };

  const sectionTitleStyle: CSSProperties = {
    fontSize: "var(--font-body-large)",
    fontWeight: 600,
    color: "var(--color-on-surface)",
    marginBottom: 12,
    paddingBottom: 8,
    borderBottom: "1px solid var(--color-surface-variant)",
  };

  const gridStyle: CSSProperties = {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
    gap: 12,
    marginBottom: 24,
  };

  const cardStyle: CSSProperties = {
    padding: "16px",
    borderRadius: 12,
    backgroundColor: "var(--color-surface-variant)",
    cursor: "pointer",
    transition: "all 0.15s ease",
    border: "1px solid transparent",
  };

  const cardHoverStyle: CSSProperties = {
    ...cardStyle,
    backgroundColor: "var(--color-primary-container)",
    borderColor: "var(--color-primary)",
    transform: "translateY(-2px)",
  };

  const inputStyle: CSSProperties = {
    width: "100%",
    padding: "10px 14px",
    borderRadius: 8,
    border: "1px solid var(--color-outline)",
    backgroundColor: "var(--color-surface-variant)",
    color: "var(--color-on-surface)",
    fontSize: "var(--font-body-medium)",
    outline: "none",
    marginBottom: 8,
  };

  const textareaStyle: CSSProperties = {
    ...inputStyle,
    minHeight: 80,
    resize: "vertical",
    fontFamily: "inherit",
  };

  const buttonStyle: CSSProperties = {
    padding: "10px 20px",
    borderRadius: 8,
    fontSize: "var(--font-body-medium)",
    fontWeight: 600,
    backgroundColor: "var(--color-primary)",
    color: "var(--color-on-primary)",
    cursor: "pointer",
    border: "none",
    transition: "opacity 0.15s ease",
  };

  return (
    <div style={containerStyle}>
      <div style={headerStyle}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button
            onClick={() => navigate("/")}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 4,
              padding: "4px 8px 4px 2px",
              borderRadius: 8,
              color: "var(--color-primary)",
              cursor: "pointer",
              backgroundColor: "transparent",
              fontSize: "var(--font-body-medium)",
              fontWeight: 400,
            }}
            aria-label="Back to chats"
          >
            <svg width="10" height="18" viewBox="0 0 10 18" fill="none">
              <path
                d="M9 1L1.5 9L9 17"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
            Back
          </button>
          <h1
            style={{
              fontSize: "var(--font-title-large)",
              fontWeight: 700,
              color: "var(--color-on-surface)",
            }}
          >
            OTP Toast Demo
          </h1>
        </div>
      </div>

      <div style={contentStyle}>
        <div
          style={{
            padding: "12px 16px",
            borderRadius: 8,
            backgroundColor: "var(--color-primary-container)",
            color: "var(--color-on-primary-container)",
            fontSize: "var(--font-body-small)",
            marginBottom: 24,
          }}
        >
          This page demonstrates the OTP toast notification system. Click any preset
          example or enter a custom code to test the liquid glass toast notification.
          The toast will appear in the top-left corner with auto-copy functionality.
        </div>

        {/* Preset Examples */}
        <div style={sectionStyle}>
          <h3 style={sectionTitleStyle}>Preset Examples</h3>
          <div style={gridStyle}>
            {presetExamples.map((example, idx) => {
              const [hovered, setHovered] = useState(false);
              return (
                <button
                  key={idx}
                  onClick={() => showOtp(example.code, example.snippet)}
                  onMouseEnter={() => setHovered(true)}
                  onMouseLeave={() => setHovered(false)}
                  style={hovered ? cardHoverStyle : cardStyle}
                >
                  <div
                    style={{
                      fontSize: "var(--font-body-small)",
                      color: "var(--color-on-surface-variant)",
                      marginBottom: 6,
                    }}
                  >
                    {example.label}
                  </div>
                  <div
                    style={{
                      fontSize: 20,
                      fontWeight: 700,
                      fontFamily: "ui-monospace, 'SF Mono', monospace",
                      color: hovered
                        ? "var(--color-on-primary-container)"
                        : "var(--color-primary)",
                      letterSpacing: "0.05em",
                      marginBottom: 8,
                    }}
                  >
                    {example.code}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-body-small)",
                      color: "var(--color-on-surface-variant)",
                      lineHeight: "1.4",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      display: "-webkit-box",
                      WebkitLineClamp: 2,
                      WebkitBoxOrient: "vertical",
                    }}
                  >
                    {example.snippet}
                  </div>
                </button>
              );
            })}
          </div>
        </div>

        {/* Custom OTP */}
        <div style={sectionStyle}>
          <h3 style={sectionTitleStyle}>Custom OTP</h3>
          <div style={{ maxWidth: 600 }}>
            <label
              style={{
                display: "block",
                fontSize: "var(--font-body-medium)",
                color: "var(--color-on-surface)",
                marginBottom: 6,
              }}
            >
              Verification Code
            </label>
            <input
              type="text"
              value={customCode}
              onChange={(e) => setCustomCode(e.target.value)}
              placeholder="Enter OTP code"
              style={inputStyle}
            />

            <label
              style={{
                display: "block",
                fontSize: "var(--font-body-medium)",
                color: "var(--color-on-surface)",
                marginBottom: 6,
                marginTop: 12,
              }}
            >
              Message Snippet
            </label>
            <textarea
              value={customSnippet}
              onChange={(e) => setCustomSnippet(e.target.value)}
              placeholder="Enter message snippet"
              style={textareaStyle}
            />

            <button
              onClick={() => showOtp(customCode, customSnippet)}
              style={buttonStyle}
              onMouseEnter={(e) => {
                e.currentTarget.style.opacity = "0.9";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.opacity = "1";
              }}
            >
              Show Custom OTP Toast
            </button>
          </div>
        </div>

        {/* Instructions */}
        <div style={sectionStyle}>
          <h3 style={sectionTitleStyle}>Testing Instructions</h3>
          <div
            style={{
              fontSize: "var(--font-body-medium)",
              color: "var(--color-on-surface)",
              lineHeight: "1.6",
            }}
          >
            <ol style={{ paddingLeft: 24, margin: 0 }}>
              <li style={{ marginBottom: 8 }}>
                Click any preset example or enter a custom code above
              </li>
              <li style={{ marginBottom: 8 }}>
                Watch the toast appear in the top-left corner with liquid glass effect
              </li>
              <li style={{ marginBottom: 8 }}>
                The code is automatically copied to your clipboard
              </li>
              <li style={{ marginBottom: 8 }}>
                Toast auto-dismisses after 5 seconds or click the × to dismiss manually
              </li>
              <li style={{ marginBottom: 8 }}>
                Toggle OTP settings in Settings → Notifications → One-Time Passwords
              </li>
            </ol>
          </div>
        </div>
      </div>
    </div>
  );
}
