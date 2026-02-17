/**
 * Attachment store - manages pending attachments and lightbox state.
 */
import { create } from "zustand";

export interface PendingAttachment {
  id: string;
  file: File;
  preview: string;
  progress: number;
  error: string | null;
  chatGuid: string;
}

export interface LightboxState {
  isOpen: boolean;
  currentIndex: number;
  images: LightboxImage[];
}

export interface LightboxImage {
  src: string;
  alt?: string;
  messageGuid?: string;
  attachmentGuid?: string;
}

interface AttachmentState {
  pendingAttachments: PendingAttachment[];
  uploadProgress: Record<string, number>;
  uploadErrors: Record<string, string>;
  lightbox: LightboxState;

  addPendingAttachment: (file: File, chatGuid: string) => PendingAttachment;
  removePendingAttachment: (id: string) => void;
  updateUploadProgress: (id: string, progress: number) => void;
  setUploadError: (id: string, error: string) => void;
  clearPendingAttachments: (chatGuid: string) => void;

  openLightbox: (images: LightboxImage[], startIndex?: number) => void;
  closeLightbox: () => void;
  nextImage: () => void;
  previousImage: () => void;
  setCurrentIndex: (index: number) => void;
}

export const useAttachmentStore = create<AttachmentState>((set, get) => ({
  pendingAttachments: [],
  uploadProgress: {},
  uploadErrors: {},
  lightbox: {
    isOpen: false,
    currentIndex: 0,
    images: [],
  },

  addPendingAttachment: (file: File, chatGuid: string) => {
    const id = `attachment-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const preview = URL.createObjectURL(file);

    const attachment: PendingAttachment = {
      id,
      file,
      preview,
      progress: 0,
      error: null,
      chatGuid,
    };

    set((state) => ({
      pendingAttachments: [...state.pendingAttachments, attachment],
    }));

    return attachment;
  },

  removePendingAttachment: (id: string) => {
    const { pendingAttachments } = get();
    const attachment = pendingAttachments.find((a) => a.id === id);

    // Revoke object URL to free memory
    if (attachment?.preview) {
      URL.revokeObjectURL(attachment.preview);
    }

    set((state) => ({
      pendingAttachments: state.pendingAttachments.filter((a) => a.id !== id),
      uploadProgress: Object.fromEntries(
        Object.entries(state.uploadProgress).filter(([key]) => key !== id)
      ),
      uploadErrors: Object.fromEntries(
        Object.entries(state.uploadErrors).filter(([key]) => key !== id)
      ),
    }));
  },

  updateUploadProgress: (id: string, progress: number) => {
    set((state) => ({
      uploadProgress: {
        ...state.uploadProgress,
        [id]: progress,
      },
      pendingAttachments: state.pendingAttachments.map((a) =>
        a.id === id ? { ...a, progress } : a
      ),
    }));
  },

  setUploadError: (id: string, error: string) => {
    set((state) => ({
      uploadErrors: {
        ...state.uploadErrors,
        [id]: error,
      },
      pendingAttachments: state.pendingAttachments.map((a) =>
        a.id === id ? { ...a, error } : a
      ),
    }));
  },

  clearPendingAttachments: (chatGuid: string) => {
    const { pendingAttachments } = get();

    // Revoke object URLs for attachments in this chat
    pendingAttachments
      .filter((a) => a.chatGuid === chatGuid)
      .forEach((a) => {
        if (a.preview) URL.revokeObjectURL(a.preview);
      });

    set((state) => ({
      pendingAttachments: state.pendingAttachments.filter(
        (a) => a.chatGuid !== chatGuid
      ),
    }));
  },

  openLightbox: (images: LightboxImage[], startIndex = 0) => {
    set({
      lightbox: {
        isOpen: true,
        currentIndex: startIndex,
        images,
      },
    });
  },

  closeLightbox: () => {
    set({
      lightbox: {
        isOpen: false,
        currentIndex: 0,
        images: [],
      },
    });
  },

  nextImage: () => {
    const { lightbox } = get();
    if (lightbox.currentIndex < lightbox.images.length - 1) {
      set({
        lightbox: {
          ...lightbox,
          currentIndex: lightbox.currentIndex + 1,
        },
      });
    }
  },

  previousImage: () => {
    const { lightbox } = get();
    if (lightbox.currentIndex > 0) {
      set({
        lightbox: {
          ...lightbox,
          currentIndex: lightbox.currentIndex - 1,
        },
      });
    }
  },

  setCurrentIndex: (index: number) => {
    const { lightbox } = get();
    if (index >= 0 && index < lightbox.images.length) {
      set({
        lightbox: {
          ...lightbox,
          currentIndex: index,
        },
      });
    }
  },
}));
