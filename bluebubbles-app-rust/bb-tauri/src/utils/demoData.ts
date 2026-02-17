/**
 * Demo data generators for showcase mode
 */

const DEMO_FIRST_NAMES = [
  "Alex", "Jordan", "Taylor", "Morgan", "Casey", "Riley", "Avery", "Quinn",
  "Sam", "Jamie", "Cameron", "Reese", "Blake", "Dakota", "Skylar", "Peyton",
  "Harper", "Rowan", "River", "Sage", "Finley", "Emerson", "Parker", "Drew"
];

const DEMO_LAST_NAMES = [
  "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
  "Martinez", "Wilson", "Anderson", "Taylor", "Thomas", "Moore", "Jackson",
  "Martin", "Lee", "Thompson", "White", "Harris", "Clark", "Lewis", "Walker"
];

const DEMO_GROUP_NAMES = [
  "Project Team", "Family", "Study Group", "Weekend Plans", "Book Club",
  "Running Crew", "Game Night", "Coffee Chat", "Work Updates", "Travel Plans",
  "Movie Night", "Fitness Squad", "Brunch Bunch", "Tech Talk", "Art Collective"
];

const DEMO_MESSAGE_SNIPPETS = [
  "That sounds great!",
  "I'll be there in 10 minutes",
  "Can we reschedule for tomorrow?",
  "Thanks for sharing that",
  "Looking forward to it!",
  "Let me know what you think",
  "Perfect timing!",
  "I completely agree",
  "Sounds like a plan",
  "See you soon!",
  "That's hilarious 😂",
  "Can't wait!",
  "Good to know",
  "I'm on my way",
  "Definitely!",
  "Maybe next week?",
  "That works for me",
  "Thanks for the update",
  "Will do!",
  "Awesome news"
];

const AVATAR_COLORS = [
  "#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8",
  "#F7DC6F", "#BB8FCE", "#85C1E2", "#F8B739", "#52B788",
  "#FF8C94", "#A8DADC", "#F1C40F", "#E74C3C", "#3498DB"
];

/**
 * Seeded random number generator
 */
function seededRandom(seed: string): number {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = ((hash << 5) - hash) + seed.charCodeAt(i);
    hash = hash & hash; // Convert to 32bit integer
  }
  const x = Math.sin(hash++) * 10000;
  return x - Math.floor(x);
}

/**
 * Generate a consistent demo name from a real name or ID
 */
export function getDemoName(realName: string | null | undefined, isGroup = false): string {
  if (!realName) return "Unknown";

  if (isGroup) {
    const index = Math.floor(seededRandom(realName) * DEMO_GROUP_NAMES.length);
    return DEMO_GROUP_NAMES[index];
  }

  const seed1 = seededRandom(realName);
  const seed2 = seededRandom(realName + "_last");

  const firstName = DEMO_FIRST_NAMES[Math.floor(seed1 * DEMO_FIRST_NAMES.length)];
  const lastName = DEMO_LAST_NAMES[Math.floor(seed2 * DEMO_LAST_NAMES.length)];

  return `${firstName} ${lastName}`;
}

/**
 * Generate initials from a demo name
 */
export function getDemoInitials(name: string): string {
  const parts = name.split(" ");
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }
  return name.slice(0, 2).toUpperCase();
}

/**
 * Generate a consistent avatar color from a name or ID
 */
export function getDemoAvatarColor(id: string): string {
  const index = Math.floor(seededRandom(id) * AVATAR_COLORS.length);
  return AVATAR_COLORS[index];
}

/**
 * Generate a demo message snippet from real message text
 */
export function getDemoMessageSnippet(realText: string | null | undefined): string {
  if (!realText) return "...";

  const index = Math.floor(seededRandom(realText) * DEMO_MESSAGE_SNIPPETS.length);
  return DEMO_MESSAGE_SNIPPETS[index];
}

/**
 * Generate SVG data URI for a circular avatar with initials
 */
export function generateDemoAvatar(name: string, id: string): string {
  const initials = getDemoInitials(name);
  const bgColor = getDemoAvatarColor(id);

  // Determine text color based on background brightness
  const r = parseInt(bgColor.slice(1, 3), 16);
  const g = parseInt(bgColor.slice(3, 5), 16);
  const b = parseInt(bgColor.slice(5, 7), 16);
  const brightness = (r * 299 + g * 587 + b * 114) / 1000;
  const textColor = brightness > 128 ? "#000000" : "#FFFFFF";

  const svg = `
    <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <circle cx="50" cy="50" r="50" fill="${bgColor}"/>
      <text
        x="50"
        y="50"
        font-family="system-ui, -apple-system, sans-serif"
        font-size="36"
        font-weight="500"
        fill="${textColor}"
        text-anchor="middle"
        dominant-baseline="central"
      >${initials}</text>
    </svg>
  `.trim();

  return `data:image/svg+xml;base64,${btoa(svg)}`;
}

/**
 * Apply demo mode transformations to a chat object
 */
export function applyDemoChatData(chat: any): any {
  if (!chat) return chat;

  const isGroup = chat.participants && chat.participants.length > 1;
  const demoName = getDemoName(chat.display_name || chat.guid, isGroup);
  const demoAvatar = generateDemoAvatar(demoName, chat.guid || "default");

  return {
    ...chat,
    display_name: demoName,
    demo_avatar: demoAvatar,
    last_message: chat.last_message ? {
      ...chat.last_message,
      text: getDemoMessageSnippet(chat.last_message.text),
    } : null,
  };
}

/**
 * Apply demo mode transformations to a message object
 */
export function applyDemoMessageData(message: any, senderName?: string): any {
  if (!message) return message;

  const demoSender = senderName ? getDemoName(senderName) : null;

  return {
    ...message,
    text: message.text ? getDemoMessageSnippet(message.text) : null,
    demo_sender_name: demoSender,
  };
}

/**
 * Apply demo mode transformations to a handle/contact object
 */
export function applyDemoHandleData(handle: any): any {
  if (!handle) return handle;

  const demoName = getDemoName(handle.display_name || handle.uncanonicalized_id);
  const demoAvatar = generateDemoAvatar(demoName, handle.address || handle.row_id?.toString() || "default");

  return {
    ...handle,
    display_name: demoName,
    demo_avatar: demoAvatar,
  };
}
