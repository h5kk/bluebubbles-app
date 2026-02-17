/**
 * FindMy store - manages Find My devices and friends location state.
 */
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface FindMyDevice {
  id: string;
  name: string;
  model: string;
  device_class: string | null;
  raw_device_model: string | null;
  battery_level: number | null;
  battery_status: string | null;
  latitude: number | null;
  longitude: number | null;
  location_timestamp: number | null;
  location_type: string | null;
  address: string | null;
  is_old_location: boolean;
  is_online: boolean;
  is_mac: boolean;
  this_device: boolean;
  lost_mode_enabled: boolean;
}

export interface FindMyFriend {
  id: string;
  handle: string; // phone/email for contact lookup
  name: string;
  latitude: number | null;
  longitude: number | null;
  address: string | null;
  last_updated: number | null;
  status: string | null;
  locating_in_progress: boolean;
}

export type FindMyTab = "devices" | "people";
export type FindMyViewMode = "map" | "list";

interface FindMyState {
  // State
  devices: Map<string, FindMyDevice>;
  friends: Map<string, FindMyFriend>;
  loadingDevices: boolean;
  loadingFriends: boolean;
  refreshing: boolean;
  error: string | null;
  selectedTab: FindMyTab;
  viewMode: FindMyViewMode;
  selectedId: string | null;
  focusPosition: [number, number] | null;

  // Actions
  fetchDevices: () => Promise<void>;
  fetchFriends: () => Promise<void>;
  refreshLocations: () => Promise<void>;
  setSelectedTab: (tab: FindMyTab) => void;
  setViewMode: (mode: FindMyViewMode) => void;
  setSelectedId: (id: string | null) => void;
  setFocusPosition: (position: [number, number] | null) => void;

  // Getters
  getAllDevices: () => FindMyDevice[];
  getAllFriends: () => FindMyFriend[];
  getDevicesWithLocation: () => FindMyDevice[];
  getFriendsWithLocation: () => FindMyFriend[];
  getSelectedItem: () => FindMyDevice | FindMyFriend | null;
}

export const useFindMyStore = create<FindMyState>((set, get) => ({
  // Initial state
  devices: new Map(),
  friends: new Map(),
  loadingDevices: true,
  loadingFriends: true,
  refreshing: false,
  error: null,
  selectedTab: "people",
  viewMode: "map",
  selectedId: null,
  focusPosition: null,

  // Fetch devices from server
  fetchDevices: async () => {
    set({ loadingDevices: true, error: null });
    try {
      const result = await invoke<FindMyDevice[]>("get_findmy_devices");
      const devicesMap = new Map<string, FindMyDevice>();
      result.forEach((device) => {
        devicesMap.set(device.id, device);
      });
      set({ devices: devicesMap, loadingDevices: false });
    } catch (err) {
      console.error("FindMy: Error fetching devices:", err);
      set({
        error: err instanceof Error ? err.message : String(err),
        loadingDevices: false,
      });
    }
  },

  // Fetch friends from server
  fetchFriends: async () => {
    set({ loadingFriends: true });
    try {
      const result = await invoke<FindMyFriend[]>("get_findmy_friends");
      const friendsMap = new Map<string, FindMyFriend>();
      result.forEach((friend) => {
        friendsMap.set(friend.id, friend);
      });
      set({ friends: friendsMap, loadingFriends: false });
    } catch (err) {
      console.warn("FindMy: Failed to fetch friends:", err);
      set({ loadingFriends: false });
    }
  },

  // Refresh locations (fetches fresh data from iCloud)
  refreshLocations: async () => {
    const { selectedTab } = get();
    set({ refreshing: true, error: null });

    try {
      if (selectedTab === "devices") {
        const result = await invoke<FindMyDevice[]>("refresh_findmy_devices");
        const devicesMap = new Map<string, FindMyDevice>();
        result.forEach((device) => {
          devicesMap.set(device.id, device);
        });
        set({ devices: devicesMap });
      } else {
        const result = await invoke<FindMyFriend[]>("refresh_findmy_friends");
        const friendsMap = new Map<string, FindMyFriend>();
        result.forEach((friend) => {
          friendsMap.set(friend.id, friend);
        });
        set({ friends: friendsMap });
      }
    } catch (err) {
      console.error("FindMy: Error during refresh:", err);
      set({ error: err instanceof Error ? err.message : String(err) });
    } finally {
      set({ refreshing: false });
    }
  },

  // UI state setters
  setSelectedTab: (tab) => {
    set({ selectedTab: tab, selectedId: null, focusPosition: null });
  },

  setViewMode: (mode) => set({ viewMode: mode }),

  setSelectedId: (id) => {
    const { selectedId: currentId } = get();
    // Toggle selection if clicking the same item
    set({ selectedId: currentId === id ? null : id });
  },

  setFocusPosition: (position) => set({ focusPosition: position }),

  // Getters
  getAllDevices: () => {
    const { devices } = get();
    return Array.from(devices.values()).sort((a, b) => {
      const timeA = a.location_timestamp ?? 0;
      const timeB = b.location_timestamp ?? 0;
      return timeB - timeA; // Most recent first (descending)
    });
  },

  getAllFriends: () => {
    const { friends } = get();
    return Array.from(friends.values()).sort((a, b) => {
      const timeA = a.last_updated ?? 0;
      const timeB = b.last_updated ?? 0;
      return timeB - timeA; // Most recent first (descending)
    });
  },

  getDevicesWithLocation: () => {
    const { devices } = get();
    return Array.from(devices.values()).filter(
      (d) => d.latitude != null && d.longitude != null
    );
  },

  getFriendsWithLocation: () => {
    const { friends } = get();
    return Array.from(friends.values()).filter(
      (f) => f.latitude != null && f.longitude != null
    );
  },

  getSelectedItem: () => {
    const { selectedId, selectedTab, devices, friends } = get();
    if (!selectedId) return null;

    if (selectedTab === "devices") {
      return devices.get(selectedId) || null;
    } else {
      return friends.get(selectedId) || null;
    }
  },
}));
