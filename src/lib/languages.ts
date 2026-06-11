export const LANG_NAMES: Record<string, string> = {
  en: 'English',
  vi: 'Vietnamese',
  ja: 'Japanese',
  ko: 'Korean',
  es: 'Spanish',
  de: 'German',
  fr: 'French',
  zh: 'Chinese',
};

export const LANG_CODES = Object.keys(LANG_NAMES);

/** Resolve a stored language code to the `{code, name}` shape the panes
 *  render. Unknown codes degrade to the uppercased code with no name. */
export function langByCode(code: string): { code: string; name: string } {
  return { code: code.toUpperCase(), name: LANG_NAMES[code] ?? '' };
}
