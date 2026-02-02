import { Translation } from "./translation";

/**
 * ようこそ、{name}さん: Welcome message
 */
export const welcome = (t: Translation, params: { name: string; }) => t.welcome.replaceAll("{name}", params.name);
