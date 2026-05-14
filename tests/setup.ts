import '@testing-library/jest-dom/vitest';

// jsdom doesn't implement matchMedia.
if (typeof window !== 'undefined' && !window.matchMedia) {
  Object.defineProperty(window, 'matchMedia', {
    value: (q: string) => ({
      matches: false, media: q,
      onchange: null,
      addListener: () => {}, removeListener: () => {},
      addEventListener: () => {}, removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  });
}
