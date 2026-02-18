/**
 * Sound playback for notifications, message send, reactions, and iMessage effects.
 * Uses real iOS audio files from public/sounds/.
 */

/** All available notification sound options with display names. */
export const NOTIFICATION_SOUNDS: { value: string; label: string; file: string }[] = [
  // iMessage defaults
  { value: "default", label: "Default (Tri-Tone)", file: "/sounds/notifications/sms-received1.mp3" },
  { value: "received-message", label: "Received Message", file: "/sounds/notifications/ReceivedMessage.mp3" },
  { value: "sms-received2", label: "Note", file: "/sounds/notifications/sms-received2.mp3" },
  { value: "sms-received3", label: "Aurora", file: "/sounds/notifications/sms-received3.mp3" },
  { value: "sms-received4", label: "Bamboo", file: "/sounds/notifications/sms-received4.mp3" },
  { value: "sms-received5", label: "Chord", file: "/sounds/notifications/sms-received5.mp3" },
  { value: "sms-received6", label: "Circles", file: "/sounds/notifications/sms-received6.mp3" },
  { value: "new-mail", label: "New Mail", file: "/sounds/notifications/new-mail.mp3" },
  // Ringtone alert tones
  { value: "apex", label: "Apex", file: "/sounds/ringtones/apex.mp3" },
  { value: "beacon", label: "Beacon", file: "/sounds/ringtones/beacon.mp3" },
  { value: "bulletin", label: "Bulletin", file: "/sounds/ringtones/bulletin.mp3" },
  { value: "chimes", label: "Chimes", file: "/sounds/ringtones/chimes.mp3" },
  { value: "constellation", label: "Constellation", file: "/sounds/ringtones/constellation.mp3" },
  { value: "cosmic", label: "Cosmic", file: "/sounds/ringtones/cosmic.mp3" },
  { value: "crystals", label: "Crystals", file: "/sounds/ringtones/crystals.mp3" },
  { value: "hillside", label: "Hillside", file: "/sounds/ringtones/hillside.mp3" },
  { value: "illuminate", label: "Illuminate", file: "/sounds/ringtones/illuminate.mp3" },
  { value: "opening", label: "Opening", file: "/sounds/ringtones/opening.mp3" },
  { value: "presto", label: "Presto", file: "/sounds/ringtones/presto.mp3" },
  { value: "radar", label: "Radar", file: "/sounds/ringtones/radar.mp3" },
  { value: "radiate", label: "Radiate", file: "/sounds/ringtones/radiate.mp3" },
  { value: "reflection", label: "Reflection", file: "/sounds/ringtones/reflection.mp3" },
  { value: "ripples", label: "Ripples", file: "/sounds/ringtones/ripples.mp3" },
  { value: "sencha", label: "Sencha", file: "/sounds/ringtones/sencha.mp3" },
  { value: "signal", label: "Signal", file: "/sounds/ringtones/signal.mp3" },
  { value: "silk", label: "Silk", file: "/sounds/ringtones/silk.mp3" },
  { value: "stargaze", label: "Stargaze", file: "/sounds/ringtones/stargaze.mp3" },
  { value: "summit", label: "Summit", file: "/sounds/ringtones/summit.mp3" },
  { value: "trill", label: "Trill", file: "/sounds/ringtones/trill.mp3" },
  { value: "twinkle", label: "Twinkle", file: "/sounds/ringtones/twinkle.mp3" },
  { value: "uplift", label: "Uplift", file: "/sounds/ringtones/uplift.mp3" },
  { value: "waves", label: "Waves", file: "/sounds/ringtones/waves.mp3" },
  // None
  { value: "none", label: "None", file: "" },
];

/** Map from iMessage effect style IDs to sound files. */
const EFFECT_SOUNDS: Record<string, string> = {
  "com.apple.MobileSMS.expressivesend.impact": "/sounds/effects/slam.mp3",
  "com.apple.MobileSMS.expressivesend.gentle": "/sounds/effects/pop.mp3",
  "com.apple.MobileSMS.expressivesend.loud": "/sounds/effects/echo-sent.mp3",
  "com.apple.messages.effect.CKConfettiEffect": "/sounds/effects/confetti.mp3",
  "com.apple.messages.effect.CKEchoEffect": "/sounds/effects/echo-sent.mp3",
  "com.apple.messages.effect.CKFireworksEffect": "/sounds/effects/fireworks.mp3",
  "com.apple.messages.effect.CKHappyBirthdayEffect": "/sounds/effects/happy-birthday.mp3",
  "com.apple.messages.effect.CKHeartEffect": "/sounds/effects/heart.mp3",
  "com.apple.messages.effect.CKLasersEffect": "/sounds/effects/lasers.mp3",
  "com.apple.messages.effect.CKShootingStarEffect": "/sounds/effects/shooting-star.mp3",
  "com.apple.messages.effect.CKSparklesEffect": "/sounds/effects/sparkles.mp3",
  "com.apple.messages.effect.CKSpotlightEffect": "/sounds/effects/spotlight.mp3",
};

// Cache Audio elements to avoid re-creating them
const audioCache = new Map<string, HTMLAudioElement>();

function getAudio(file: string): HTMLAudioElement {
  let audio = audioCache.get(file);
  if (!audio) {
    audio = new Audio(file);
    audioCache.set(file, audio);
  }
  return audio;
}

function playFile(file: string) {
  if (!file) return;
  const audio = getAudio(file);
  audio.currentTime = 0;
  audio.play().catch(() => {
    // Playback may fail if user hasn't interacted yet
  });
}

/**
 * Play a notification sound by style value.
 * @param style - one of the NOTIFICATION_SOUNDS value strings, or "none"
 */
export function playNotificationSound(style: string = "default") {
  if (style === "none") return;
  const entry = NOTIFICATION_SOUNDS.find((s) => s.value === style);
  if (entry?.file) {
    playFile(entry.file);
  }
}

/** Play the message-sent sound. */
export function playSentSound() {
  playFile("/sounds/ui/sent-message.mp3");
}

/** Play reaction (tapback) sound. */
export function playReactionSound(sent: boolean = true) {
  playFile(sent ? "/sounds/ui/reaction-sent.mp3" : "/sounds/ui/reaction-received.mp3");
}

/** Play iMessage effect sound based on the expressive_send_style_id. */
export function playEffectSound(effectId: string | null) {
  if (!effectId) return;
  const file = EFFECT_SOUNDS[effectId];
  if (file) playFile(file);
}
