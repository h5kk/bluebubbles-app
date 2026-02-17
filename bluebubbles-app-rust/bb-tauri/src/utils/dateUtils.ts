/**
 * Date/timestamp utilities for BlueBubbles.
 *
 * The BB server can return dates in three formats:
 * 1. Epoch milliseconds (13 digits, e.g. 1700000000000)
 * 2. Apple Core Data timestamp (seconds since 2001-01-01, ~9-10 digits, e.g. 700000000)
 * 3. ISO 8601 string (e.g. "2024-01-15T10:30:00Z")
 *
 * Apple Core Data epoch starts at 2001-01-01 00:00:00 UTC.
 * Offset from Unix epoch: 978307200 seconds.
 */

/** Seconds between Unix epoch (1970-01-01) and Apple Core Data epoch (2001-01-01). */
const APPLE_EPOCH_OFFSET = 978307200;

/**
 * Convert a BB server timestamp (any format) to a JS Date.
 * Returns null if the value is null, undefined, or unparseable.
 */
export function parseBBDate(value: string | number | null | undefined): Date | null {
  if (value == null || value === "") return null;

  // If it's already a number or a numeric string, figure out which epoch
  const num = typeof value === "number" ? value : Number(value);

  if (!isNaN(num) && num !== 0) {
    // Heuristic to distinguish the three numeric cases:
    // - Epoch milliseconds: 13+ digits (>= 1_000_000_000_000), dates from ~2001 onward
    // - Epoch seconds: 10 digits (~1_000_000_000 to 9_999_999_999), but we won't
    //   typically see plain Unix seconds from BB server
    // - Apple Core Data: smaller numbers (seconds since 2001-01-01). A value of
    //   700000000 = ~2023. These are typically < 1_000_000_000_000.
    //
    // Key insight: Apple Core Data timestamps for recent dates (2001-2040) range
    // roughly from 0 to ~1_200_000_000. Unix epoch seconds for the same range are
    // ~978_307_200 to ~2_208_988_800. Epoch milliseconds are >= 1e12.
    //
    // Strategy:
    // 1. If >= 1e12 -> epoch milliseconds
    // 2. If the number, interpreted as Unix seconds, gives a date before 2001 AND
    //    interpreted as Apple Core Data gives a reasonable date -> Apple Core Data
    // 3. Otherwise -> treat as Unix seconds (multiply by 1000)

    if (num >= 1e12) {
      // Epoch milliseconds
      return new Date(num);
    }

    // Try interpreting as Apple Core Data timestamp
    const asAppleMs = (num + APPLE_EPOCH_OFFSET) * 1000;
    const asUnixMs = num * 1000;

    // Apple Core Data timestamps for dates between 2001 and ~2040 are between 0 and ~1.2e9
    // Unix seconds for dates between 2001 and ~2040 are between ~978307200 and ~2.2e9
    // If the number is less than APPLE_EPOCH_OFFSET (~978M), it's definitely Apple Core Data
    // because as Unix seconds it would be before 2001 (before iMessage existed)
    if (num < APPLE_EPOCH_OFFSET) {
      return new Date(asAppleMs);
    }

    // For numbers between ~978M and ~1.2e9, both interpretations give dates in the 2000s-2030s range.
    // We check: if interpreting as Unix seconds gives a date before 2001, it must be Apple Core Data.
    const asUnixDate = new Date(asUnixMs);
    if (asUnixDate.getFullYear() < 2001) {
      return new Date(asAppleMs);
    }

    // Default: treat as Unix seconds
    return asUnixDate;
  }

  // Not numeric - try parsing as a date string (ISO 8601, etc.)
  if (typeof value === "string") {
    const parsed = new Date(value);
    if (!isNaN(parsed.getTime())) return parsed;
  }

  return null;
}

/**
 * Convert a BB server timestamp to epoch milliseconds.
 * Returns null if unparseable.
 */
export function parseBBDateMs(value: string | number | null | undefined): number | null {
  const d = parseBBDate(value);
  return d ? d.getTime() : null;
}
