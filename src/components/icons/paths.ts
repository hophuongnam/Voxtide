export const PATHS = {
  mic:     'M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3z M5 11a7 7 0 0 0 14 0 M12 18v3 M8 21h8',
  speaker: 'M3 10v4h4l5 4V6l-5 4H3z M16 8a5 5 0 0 1 0 8 M19.5 5a9 9 0 0 1 0 14',
  overlay: 'M3 5h12v10H3z M9 9h12v10H9z',
  cog:     'M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z M19.4 13a7.5 7.5 0 0 0 0-2l2-1.5-2-3.4-2.4.8a7.5 7.5 0 0 0-1.7-1L14.8 3h-5.6l-.5 2.6a7.5 7.5 0 0 0-1.7 1l-2.4-.8-2 3.4 2 1.5a7.5 7.5 0 0 0 0 2l-2 1.5 2 3.4 2.4-.8a7.5 7.5 0 0 0 1.7 1l.5 2.6h5.6l.5-2.6a7.5 7.5 0 0 0 1.7-1l2.4.8 2-3.4-2-1.5z',
  search:  'M10.5 4a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13z M20 20l-4.5-4.5',
  play:    'M6 4l14 8-14 8V4z',
  swap:    'M7 7h13l-3-3 M17 17H4l3 3',
  trash:   'M4 7h16 M9 7V4h6v3 M6 7l1 13h10l1-13 M10 11v6 M14 11v6',
  chevron: 'M6 9l6 6 6-6',
  close:   'M6 6l12 12 M18 6L6 18',
  plus:    'M12 5v14 M5 12h14',
  arrow:   'M5 12h14 M14 7l5 5-5 5',
  key:     'M14 8a4 4 0 1 1-4 4 M14 8l6 0 m0 0v3 m-3-3v3 M10 12L3 19v2h2l1-1h2v-2h2l1-1',
} as const;

export type IconName = keyof typeof PATHS;
