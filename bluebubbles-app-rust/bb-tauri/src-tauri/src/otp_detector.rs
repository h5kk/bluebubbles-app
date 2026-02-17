//! OTP (One-Time Password) detection module.
//!
//! Detects verification codes and OTPs in message text using regex patterns.
//! Optimized for <1ms per message detection.

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    // Pattern 1: "Your code is 123456" or "verification code: 123456" or "Backup code: 123456"
    static ref CODE_WITH_PREFIX: Regex = Regex::new(
        r"(?i)(?:your|the|verification|security|access|authentication|login|otp|one-time|backup)\s+(?:code|password|otp|pin)\s*(?:is|:|are)?\s*(\d{4,8})\b"
    ).unwrap();

    // Pattern 2: "123456 is your code"
    static ref CODE_WITH_SUFFIX: Regex = Regex::new(
        r"(?i)\b(\d{4,8})\s+(?:is|as)\s+(?:your|the|a|an)\s+(?:verification|security|access|authentication|login|otp|one-time)?\s*(?:code|password|otp|pin)\b"
    ).unwrap();

    // Pattern 3: Apple specific format: "Your Apple ID Code is: 123456"
    static ref APPLE_FORMAT: Regex = Regex::new(
        r"(?i)(?:apple|icloud)\s+(?:id\s+)?code\s*(?:is)?\s*:\s*(\d{4,8})\b"
    ).unwrap();

    // Pattern 4: Google specific format: "G-123456 is your Google verification code"
    static ref GOOGLE_FORMAT: Regex = Regex::new(
        r"(?i)G-(\d{6})\b"
    ).unwrap();

    // Pattern 5: Standalone code in brackets or parentheses: "Use code (123456)"
    static ref BRACKETED_CODE: Regex = Regex::new(
        r"(?i)(?:use|enter|input)?\s*(?:code|otp|pin)?\s*[\(\[\{](\d{4,8})[\)\]\}]"
    ).unwrap();

    // Pattern 6: Standalone numeric code (4-8 digits) with word boundaries
    // Only matches if not part of phone numbers or dates
    static ref STANDALONE_CODE: Regex = Regex::new(
        r"(?i)(?:^|\s|:|\.|\,)\s*(\d{4,8})\s*(?:\s|$|\.|\,)"
    ).unwrap();
}

/// Detected OTP code with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OtpDetection {
    /// The detected code (numeric only).
    pub code: String,
    /// The pattern that matched (for debugging/logging).
    pub pattern: OtpPattern,
    /// Position in the text where the code was found.
    pub position: usize,
}

/// Type of pattern that detected the OTP.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OtpPattern {
    CodeWithPrefix,
    CodeWithSuffix,
    AppleFormat,
    GoogleFormat,
    BracketedCode,
    StandaloneCode,
}

impl OtpPattern {
    /// Get a human-readable description of the pattern.
    pub fn description(&self) -> &'static str {
        match self {
            OtpPattern::CodeWithPrefix => "Code with prefix (e.g., 'Your code is 123456')",
            OtpPattern::CodeWithSuffix => "Code with suffix (e.g., '123456 is your code')",
            OtpPattern::AppleFormat => "Apple format (e.g., 'Apple ID Code is: 123456')",
            OtpPattern::GoogleFormat => "Google format (e.g., 'G-123456')",
            OtpPattern::BracketedCode => "Bracketed code (e.g., 'Use code (123456)')",
            OtpPattern::StandaloneCode => "Standalone numeric code",
        }
    }
}

/// Detect OTP codes in message text.
///
/// Returns the first detected code, prioritizing more specific patterns over generic ones.
/// Performance: <1ms per message on average hardware.
pub fn detect_otp(text: &str) -> Option<OtpDetection> {
    if text.is_empty() {
        return None;
    }

    // Try patterns in order of specificity (most specific first)

    // 1. Apple format
    if let Some(captures) = APPLE_FORMAT.captures(text) {
        if let Some(m) = captures.get(1) {
            return Some(OtpDetection {
                code: m.as_str().to_string(),
                pattern: OtpPattern::AppleFormat,
                position: m.start(),
            });
        }
    }

    // 2. Google format
    if let Some(captures) = GOOGLE_FORMAT.captures(text) {
        if let Some(m) = captures.get(1) {
            return Some(OtpDetection {
                code: m.as_str().to_string(),
                pattern: OtpPattern::GoogleFormat,
                position: m.start(),
            });
        }
    }

    // 3. Code with prefix (most common)
    if let Some(captures) = CODE_WITH_PREFIX.captures(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            // Validate length (4-8 digits)
            if (4..=8).contains(&code.len()) {
                return Some(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::CodeWithPrefix,
                    position: m.start(),
                });
            }
        }
    }

    // 4. Code with suffix
    if let Some(captures) = CODE_WITH_SUFFIX.captures(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            if (4..=8).contains(&code.len()) {
                return Some(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::CodeWithSuffix,
                    position: m.start(),
                });
            }
        }
    }

    // 5. Bracketed code
    if let Some(captures) = BRACKETED_CODE.captures(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            if (4..=8).contains(&code.len()) {
                return Some(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::BracketedCode,
                    position: m.start(),
                });
            }
        }
    }

    // 6. Standalone code (least specific, only if no other pattern matched)
    // Additional validation: must contain OTP-related keywords nearby
    let lower = text.to_lowercase();
    if lower.contains("code")
        || lower.contains("verification")
        || lower.contains("otp")
        || lower.contains("pin")
        || lower.contains("security")
        || lower.contains("verify")
        || lower.contains("password")
        || lower.contains("use")
        || lower.contains("enter") {
        if let Some(captures) = STANDALONE_CODE.captures(text) {
            if let Some(m) = captures.get(1) {
                let code = m.as_str();
                if (4..=8).contains(&code.len()) && !is_likely_date_or_phone(code, text) {
                    return Some(OtpDetection {
                        code: code.to_string(),
                        pattern: OtpPattern::StandaloneCode,
                        position: m.start(),
                    });
                }
            }
        }
    }

    None
}

/// Heuristic to filter out dates and phone numbers.
fn is_likely_date_or_phone(code: &str, text: &str) -> bool {
    // If the code is 4 digits and preceded by a slash or dash, it's likely a year
    if code.len() == 4 && text.contains(&format!("/{}", code)) {
        return true;
    }
    if code.len() == 4 && text.contains(&format!("-{}", code)) {
        return true;
    }

    // If surrounded by dashes or parentheses, might be a phone number
    if text.contains(&format!("-{}-", code)) || text.contains(&format!("({})", code)) {
        return true;
    }

    false
}

/// Detect all OTP codes in message text (returns all matches).
///
/// Useful for testing or when multiple codes might be present.
pub fn detect_all_otps(text: &str) -> Vec<OtpDetection> {
    let mut detections = Vec::new();

    if text.is_empty() {
        return detections;
    }

    // Check all patterns
    for captures in APPLE_FORMAT.captures_iter(text) {
        if let Some(m) = captures.get(1) {
            detections.push(OtpDetection {
                code: m.as_str().to_string(),
                pattern: OtpPattern::AppleFormat,
                position: m.start(),
            });
        }
    }

    for captures in GOOGLE_FORMAT.captures_iter(text) {
        if let Some(m) = captures.get(1) {
            detections.push(OtpDetection {
                code: m.as_str().to_string(),
                pattern: OtpPattern::GoogleFormat,
                position: m.start(),
            });
        }
    }

    for captures in CODE_WITH_PREFIX.captures_iter(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            if (4..=8).contains(&code.len()) {
                detections.push(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::CodeWithPrefix,
                    position: m.start(),
                });
            }
        }
    }

    for captures in CODE_WITH_SUFFIX.captures_iter(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            if (4..=8).contains(&code.len()) {
                detections.push(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::CodeWithSuffix,
                    position: m.start(),
                });
            }
        }
    }

    for captures in BRACKETED_CODE.captures_iter(text) {
        if let Some(m) = captures.get(1) {
            let code = m.as_str();
            if (4..=8).contains(&code.len()) {
                detections.push(OtpDetection {
                    code: code.to_string(),
                    pattern: OtpPattern::BracketedCode,
                    position: m.start(),
                });
            }
        }
    }

    // Include standalone codes if relevant keywords are present
    let lower = text.to_lowercase();
    if lower.contains("code")
        || lower.contains("verification")
        || lower.contains("otp")
        || lower.contains("pin")
        || lower.contains("security")
        || lower.contains("verify")
        || lower.contains("password") {
        for captures in STANDALONE_CODE.captures_iter(text) {
            if let Some(m) = captures.get(1) {
                let code = m.as_str();
                if (4..=8).contains(&code.len()) && !is_likely_date_or_phone(code, text) {
                    // Check if this code was already detected by a more specific pattern
                    let already_detected = detections.iter().any(|d| d.code == code);
                    if !already_detected {
                        detections.push(OtpDetection {
                            code: code.to_string(),
                            pattern: OtpPattern::StandaloneCode,
                            position: m.start(),
                        });
                    }
                }
            }
        }
    }

    detections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_with_prefix() {
        let text = "Your verification code is 123456";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "123456");
        assert_eq!(detection.pattern, OtpPattern::CodeWithPrefix);
    }

    #[test]
    fn test_code_with_suffix() {
        let text = "123456 is your verification code";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "123456");
        assert_eq!(detection.pattern, OtpPattern::CodeWithSuffix);
    }

    #[test]
    fn test_apple_format() {
        let text = "Your Apple ID Code is: 654321";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "654321");
        assert_eq!(detection.pattern, OtpPattern::AppleFormat);
    }

    #[test]
    fn test_google_format() {
        let text = "G-123456 is your Google verification code";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "123456");
        assert_eq!(detection.pattern, OtpPattern::GoogleFormat);
    }

    #[test]
    fn test_bracketed_code() {
        let text = "Use code (987654) to sign in";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "987654");
        assert_eq!(detection.pattern, OtpPattern::BracketedCode);
    }

    #[test]
    fn test_standalone_code() {
        let text = "Your security code: 4567";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "4567");
    }

    #[test]
    fn test_no_otp() {
        let text = "Hey, how are you doing today?";
        assert!(detect_otp(text).is_none());
    }

    #[test]
    fn test_phone_number_rejection() {
        let text = "Call me at 555-1234 tomorrow";
        // Should not detect phone numbers as OTPs
        assert!(detect_otp(text).is_none());
    }

    #[test]
    fn test_date_rejection() {
        let text = "The meeting is on 12/2024";
        // Should not detect years as OTPs
        assert!(detect_otp(text).is_none());
    }

    #[test]
    fn test_multiple_codes() {
        let text = "Your code is 123456. Backup code: 789012";
        let detections = detect_all_otps(text);
        assert_eq!(detections.len(), 2);
        assert_eq!(detections[0].code, "123456");
        assert_eq!(detections[1].code, "789012");
    }

    #[test]
    fn test_case_insensitive() {
        let text = "YOUR VERIFICATION CODE IS 456789";
        let detection = detect_otp(text).unwrap();
        assert_eq!(detection.code, "456789");
    }

    #[test]
    fn test_various_lengths() {
        assert_eq!(detect_otp("Code: 1234").unwrap().code, "1234"); // 4 digits
        assert_eq!(detect_otp("Code: 12345").unwrap().code, "12345"); // 5 digits
        assert_eq!(detect_otp("Code: 123456").unwrap().code, "123456"); // 6 digits
        assert_eq!(detect_otp("Code: 1234567").unwrap().code, "1234567"); // 7 digits
        assert_eq!(detect_otp("Code: 12345678").unwrap().code, "12345678"); // 8 digits
        assert!(detect_otp("Code: 123").is_none()); // Too short
        assert!(detect_otp("Code: 123456789").is_none()); // Too long
    }

    #[test]
    fn test_empty_text() {
        assert!(detect_otp("").is_none());
    }

    #[test]
    fn test_real_world_examples() {
        // Real-world Apple example
        assert_eq!(
            detect_otp("Your Apple ID Code is: 123456. Don't share it with anyone.").unwrap().code,
            "123456"
        );

        // Real-world Google example
        assert_eq!(
            detect_otp("G-654321 is your Google verification code.").unwrap().code,
            "654321"
        );

        // Real-world generic example
        assert_eq!(
            detect_otp("Use 987654 to verify your phone number.").unwrap().code,
            "987654"
        );

        // Real-world banking example
        assert_eq!(
            detect_otp("Your one-time password is 112233").unwrap().code,
            "112233"
        );
    }

    #[test]
    fn test_performance() {
        use std::time::Instant;

        let test_messages = vec![
            "Your verification code is 123456",
            "G-654321 is your Google verification code",
            "Your Apple ID Code is: 789012",
            "Hey, how are you doing? Let's meet at 5pm tomorrow.",
            "Use code (456789) to sign in to your account",
            "The meeting is scheduled for 12/15/2024 at 3pm",
            "Call me at 555-1234 when you get this message",
            "Your one-time password is 998877. Valid for 5 minutes.",
            "Just checking in to see how the project is going!",
            "Backup code: 112233 in case you lose access to your device",
        ];

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            for msg in &test_messages {
                let _ = detect_otp(msg);
            }
        }

        let elapsed = start.elapsed();
        let avg_per_message = elapsed / (iterations * test_messages.len() as u32);

        println!("Average detection time: {:?} per message", avg_per_message);
        println!("Total time for {} detections: {:?}", iterations * test_messages.len() as u32, elapsed);

        // Performance requirement: <1ms per message
        assert!(
            avg_per_message.as_micros() < 1000,
            "Detection took {:?} per message, which exceeds 1ms requirement",
            avg_per_message
        );
    }
}
