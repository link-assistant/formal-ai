// Type declarations for the `lino-i18n` package.

export interface I18nOptions {
  /** Translation catalogues keyed by locale code. */
  locales?: Record<string, Record<string, string>>;
  /** Locale used when no `setLocale` has been called. */
  defaultLocale?: string;
  /** Fallback locale (or array of locales) used when a key is missing. */
  fallback?: string | string[];
  /** Callback invoked when a key is missing in every locale. */
  onMissingKey?: (info: {
    key: string;
    params: Record<string, unknown>;
    options: TOptions;
  }) => string | void;
  /** Interpolation tokens; currently informational only. */
  interpolation?: { prefix?: string; suffix?: string };
}

export interface TOptions {
  /** Force a specific locale for this call only. */
  locale?: string;
  /** Gender / context suffix (resolves `key_context`). */
  context?: string;
  /** Default value returned when the key is missing. */
  defaultValue?: string;
}

export interface TParams extends Record<string, unknown> {
  /** Numeric count used for plural resolution. */
  count?: number;
  /** Context suffix that overrides `TOptions.context`. */
  context?: string;
  /** Default value used when the key is missing. */
  defaultValue?: string;
}

export interface I18nInstance {
  t(key: string, params?: TParams, options?: TOptions): string;
  has(key: string, locale?: string): boolean;
  getLocale(): string;
  setLocale(locale: string): void;
  getFallbacks(): string[];
  listLocales(): string[];
  addLocale(locale: string, translations: Record<string, string>): void;
  loadLocale(locale: string, text: string): Promise<string>;
  loadLocaleFile(filePath: string): Promise<string>;
  loadDirectory(directory: string): Promise<string[]>;
  interpolation: { prefix?: string; suffix?: string };
}

export declare function createI18n(options?: I18nOptions): I18nInstance;

export declare function parseLinoCatalog(text: string): {
  locale: string | null;
  translations: Record<string, string>;
};

export declare function parseLinoCatalogs(
  text: string
): Array<{ locale: string | null; translations: Record<string, string> }>;

export declare function formatLinoCatalog(
  locale: string,
  translations: Record<string, string>,
  options?: { style?: 'nested' | 'flat' }
): string;

export declare function formatLinoCatalogs(
  catalogues:
    | Record<string, Record<string, string>>
    | Array<{ locale: string; translations: Record<string, string> }>,
  options?: { style?: 'nested' | 'flat' }
): string;

export declare function loadLocaleFromString(
  locale: string,
  text: string
): Promise<{ locale: string; translations: Record<string, string> }>;

export declare function loadLocaleFromFile(
  filePath: string
): Promise<{ locale: string; translations: Record<string, string> }>;

export declare function loadLocalesFromFile(
  filePath: string
): Promise<Array<{ locale: string; translations: Record<string, string> }>>;

export declare function loadLocalesFromDirectory(
  directory: string
): Promise<Record<string, Record<string, string>>>;

export declare function interpolate(
  template: string,
  params: Record<string, unknown> | undefined
): string;

export declare function pluralSuffix(
  locale: string,
  count: number
): string | null;

export declare function resolveKey(
  table: Record<string, string>,
  key: string,
  options?: { count?: number; context?: string; locale?: string }
): string | undefined;

export declare function fromI18next(
  input: unknown,
  options?: { locale?: string; defaultLocale?: string }
): Record<string, Record<string, string>>;

export declare function fromI18nJs(
  input: unknown,
  options?: { locale?: string; defaultLocale?: string }
): Record<string, Record<string, string>>;

export declare function fromReactIntl(
  input: unknown,
  options?: { locale?: string; defaultLocale?: string }
): Record<string, Record<string, string>>;
