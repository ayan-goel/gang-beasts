export function storedChoice<T extends string>(key: string, choices: readonly T[], fallback: T) {
  const listeners = new Set<() => void>();
  let current: T | undefined;
  let unsaved = false;

  function read() {
    try {
      const stored = localStorage.getItem(key);
      return choices.find(choice => choice === stored) ?? fallback;
    } catch { return current ?? fallback; }
  }

  function publish(choice: T) {
    if (current === choice) return;
    current = choice;
    for (const listener of listeners) listener();
  }

  function onStorage(event: StorageEvent) {
    if (event.key === key || event.key === null) {
      unsaved = false;
      publish(read());
    }
  }

  return {
    value: () => current ??= read(),
    choose(choice: T) {
      try { localStorage.setItem(key, choice); unsaved = false; } catch { unsaved = true; }
      publish(choice);
    },
    subscribe(listener: () => void) {
      const first = !listeners.size;
      listeners.add(listener);
      if (first) {
        window.addEventListener('storage', onStorage);
        if (!unsaved) publish(read());
      }
      return () => {
        listeners.delete(listener);
        if (!listeners.size) window.removeEventListener('storage', onStorage);
      };
    },
  };
}
