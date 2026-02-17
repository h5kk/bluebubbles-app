/**
 * OTP Toast Context
 *
 * Manages the global state for OTP toast notifications.
 * Provides methods to show/dismiss OTP codes detected from messages.
 */
import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import type { OtpToastData } from "@/components/OtpToast";

interface OtpToastContextValue {
  /** Current OTP data being displayed (null if none) */
  otpData: OtpToastData | null;
  /** Show an OTP code in the toast */
  showOtp: (code: string, snippet: string) => void;
  /** Dismiss the current OTP toast */
  dismissOtp: () => void;
}

const OtpToastContext = createContext<OtpToastContextValue | null>(null);

interface OtpToastProviderProps {
  children: ReactNode;
}

export function OtpToastProvider({ children }: OtpToastProviderProps) {
  const [otpData, setOtpData] = useState<OtpToastData | null>(null);

  const showOtp = useCallback((code: string, snippet: string) => {
    setOtpData({
      code,
      snippet,
      timestamp: Date.now(),
    });
  }, []);

  const dismissOtp = useCallback(() => {
    setOtpData(null);
  }, []);

  return (
    <OtpToastContext.Provider value={{ otpData, showOtp, dismissOtp }}>
      {children}
    </OtpToastContext.Provider>
  );
}

/**
 * Hook to access OTP toast context.
 * Throws if used outside of OtpToastProvider.
 */
export function useOtpToast() {
  const context = useContext(OtpToastContext);
  if (!context) {
    throw new Error("useOtpToast must be used within OtpToastProvider");
  }
  return context;
}
