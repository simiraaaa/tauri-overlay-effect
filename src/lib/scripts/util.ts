type DebouncedFunction<T extends (...args: unknown[]) => unknown> =
  ((...args: Parameters<T>) => void) & { cancel: () => void };

export const debounce = <T extends (...args: unknown[]) => unknown>(
  func: T,
  ms: number,
): DebouncedFunction<T> => {
  let id: ReturnType<typeof setTimeout> | null = null;

  const debounced = (...args: Parameters<T>) => {
    if (id !== null) {
      clearTimeout(id);
    }
    id = setTimeout(() => {
      func(...args);
    }, ms);
  };

  return Object.assign(debounced, {
    cancel: () => {
      if (id !== null) {
        clearTimeout(id);
      }
    },
  });
};
