export interface ShortcutEntry {
  action: () => void;
  ctrl?: boolean;
  description: string;
  key: string;
  meta?: boolean;
  shift?: boolean;
}

/**
 * Creates a keyboard shortcut registry.
 * Call `handleKeyDown` from a global keydown listener.
 */
export function createShortcutRegistry() {
  const shortcuts: ShortcutEntry[] = [];

  function register(entry: ShortcutEntry) {
    shortcuts.push(entry);
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Skip if typing in an input
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") {
      return;
    }

    for (const s of shortcuts) {
      const ctrlMatch = (s.ctrl ?? false) === (e.ctrlKey || e.metaKey);
      const shiftMatch = (s.shift ?? false) === e.shiftKey;
      const keyMatch = e.key.toLowerCase() === s.key.toLowerCase();

      if (keyMatch && ctrlMatch && shiftMatch) {
        e.preventDefault();
        s.action();
        return;
      }
    }
  }

  function getAll(): readonly ShortcutEntry[] {
    return shortcuts;
  }

  return { register, handleKeyDown, getAll };
}
