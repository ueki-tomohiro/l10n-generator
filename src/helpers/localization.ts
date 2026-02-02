const locales = ["ja", "en"] as const;
export type SupportLocale = typeof locales[number];
