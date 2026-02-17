/**
 * OTP Detection Hook
 *
 * Listens for OTP detection events from the Tauri backend and
 * automatically displays them in the OTP toast notification.
 */
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useOtpToast } from "@/contexts/OtpToastContext";
import { useSettingsStore } from "@/store/settingsStore";

/**
 * Payload structure for OTP detection events emitted from Rust.
 */
interface OtpDetectionPayload {
  /** The detected OTP code (e.g., "123456") */
  code: string;
  /** Text snippet from the message containing the OTP */
  snippet: string;
}

/**
 * Hook that listens for OTP detection events from Tauri backend.
 * Automatically shows OTP codes in toast when detected, if enabled in settings.
 */
export function useOtpDetection() {
  const { showOtp } = useOtpToast();
  const { settings } = useSettingsStore();
  const otpDetectionEnabled = settings["otpDetection"] !== "false";
  const otpAutoCopyEnabled = settings["otpAutoCopy"] !== "false";

  useEffect(() => {
    if (!otpDetectionEnabled) {
      return;
    }

    let unlisten: UnlistenFn | undefined;

    // Listen for OTP detection events from Tauri backend
    listen<OtpDetectionPayload>("otp-detected", (event) => {
      const { code, snippet } = event.payload;

      // Show OTP in toast (which will auto-copy if enabled)
      if (code) {
        showOtp(code, snippet || "");
      }
    }).then((fn) => {
      unlisten = fn;
    }).catch((err) => {
      console.error("failed to listen for OTP events:", err);
    });

    // Cleanup listener on unmount
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [otpDetectionEnabled, otpAutoCopyEnabled, showOtp]);
}
