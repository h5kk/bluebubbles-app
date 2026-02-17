/**
 * Contact store - manages contact avatar data.
 *
 * Loads all contact avatars in a single bulk IPC call and provides
 * a lookup function for the Avatar component. Avatars are cached
 * in memory and only re-fetched on explicit refresh.
 */
import { create } from "zustand";
import { tauriGetAllContactAvatars, tauriSyncContactAvatars } from "@/hooks/useTauri";

/** Normalize a phone number by stripping non-digit chars (except leading +). */
function normalizeAddress(addr: string): string {
  return addr.replace(/[^0-9+]/g, "");
}

interface ContactState {
  /** Map of address -> data URI for avatars. Includes multiple key forms per contact. */
  avatars: Record<string, string>;
  /** Whether avatars have been loaded at least once. */
  loaded: boolean;
  /** Whether a load is currently in progress. */
  loading: boolean;

  /**
   * Load all contact avatars from the local database in a single IPC call.
   * This is fast since it reads from the already-synced local DB.
   */
  loadAvatars: () => Promise<void>;

  /**
   * Sync avatars from the remote server, then reload from local DB.
   * Call this after initial connection to ensure avatars are up to date.
   */
  syncAndLoadAvatars: () => Promise<void>;

  /**
   * Look up an avatar data URI by address (phone or email).
   * Tries the raw address first, then normalized forms.
   * Returns null if no avatar is found.
   */
  getAvatar: (address: string) => string | null;
}

export const useContactStore = create<ContactState>((set, get) => ({
  avatars: {},
  loaded: false,
  loading: false,

  loadAvatars: async () => {
    if (get().loading) return;
    set({ loading: true });
    try {
      const avatars = await tauriGetAllContactAvatars();
      const count = Object.keys(avatars).length;
      console.log(`[contactStore] loadAvatars: ${count} avatar entries loaded from DB`);
      if (count > 0) {
        const sample = Object.keys(avatars).slice(0, 3);
        console.log(`[contactStore] sample keys:`, sample);
        console.log(`[contactStore] sample value length:`, avatars[sample[0]]?.length);
      }
      set({ avatars, loaded: true, loading: false });
    } catch (err) {
      console.error("[contactStore] failed to load contact avatars:", err);
      set({ loading: false, loaded: true });
    }
  },

  syncAndLoadAvatars: async () => {
    if (get().loading) return;
    set({ loading: true });
    try {
      console.log("[contactStore] syncing avatars from server...");
      const synced = await tauriSyncContactAvatars();
      console.log(`[contactStore] server sync complete: ${synced} avatars synced`);
      // Then load from local DB
      const avatars = await tauriGetAllContactAvatars();
      const count = Object.keys(avatars).length;
      console.log(`[contactStore] after sync: ${count} avatar entries loaded from DB`);
      set({ avatars, loaded: true, loading: false });
    } catch (err) {
      console.error("[contactStore] failed to sync contact avatars:", err);
      set({ loading: false });
      // Even if sync fails, try loading whatever is in local DB
      try {
        const avatars = await tauriGetAllContactAvatars();
        set({ avatars, loaded: true });
      } catch {
        // DB read also failed, nothing we can do
      }
    }
  },

  getAvatar: (address: string): string | null => {
    const { avatars } = get();
    // Direct match
    if (avatars[address]) return avatars[address];
    // Lowercase match (for emails)
    const lower = address.toLowerCase();
    if (avatars[lower]) return avatars[lower];
    // Normalized phone match
    const normalized = normalizeAddress(address);
    if (avatars[normalized]) return avatars[normalized];
    return null;
  },
}));
