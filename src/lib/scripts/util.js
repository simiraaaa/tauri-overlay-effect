/**
 * 
 * @param {(...args: any) => any} func 
 * @param {number} ms 
 * @returns {((...args: any) => any) & { cancel: () => void }}
 */
export function debounce(func, ms) {
  /** @type {NodeJS.Timeout} */
  let id;
  return Object.assign((/** @type {any[]} */ ...args) => {
    clearTimeout(id);
    id = setTimeout(() => func(...args), ms);
  }, {
    cancel: () => clearTimeout(id),
  });
}
