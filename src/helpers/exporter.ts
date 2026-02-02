import fs from "fs";
import camelCase from "lodash/camelCase";
import { SupportLocale } from "./localization";

const parseInputParameter = (text: string) => {
  if (!text) return undefined;
  const matches = text.match(/\{\w+\}/g);
  if (!matches || matches.length === 0) return undefined;
  const params = matches.map((t) => t.match(/\w+/)).flatMap((a) => a) as string[];

  var place: {
    [key: string]: any;
  } = {};
  params.forEach((p) => {
    place[p] = {
      type: "String",
      example: p,
    };
  });

  return place;
};

const createInputParameterFunction = (key: string, text: string) => {
  if (!text) return undefined;
  const matches = text.match(/\{\w+\}/g);
  if (!matches || matches.length === 0) return undefined;
  const params = matches.map((t) => t.match(/\w+/)).flatMap((a) => a) as string[];

  return `export const ${camelCase(key)} = (t: Translation, params: { ${params
    .map((p) => `${p}: string;`)
    .join(" ")} }) => t.${key}${params.map((p) => `.replaceAll("{${p}}", params.${p})`).join("")};`;
};

export const createL10nFile = async (localizePath: string, values: string[][], locale: SupportLocale) => {
  const labels = values.pop();
  if (!labels) return;
  const localColumn = labels.findIndex((l) => l == locale) - 2;

  const lines = values
    .map((line: string[]) => {
      const [key, description, ...params] = line;
      var value: any = {};
      const word = params[localColumn];
      value[key] = word;
      const placeholders = parseInputParameter(word);
      value[`@${key}`] = {
        description,
        placeholders,
      };
      return value;
    })
    .reduce((previousValue: any, currentValue: any) => {
      return { ...previousValue, ...currentValue };
    });

  const ja = { "@@locale": locale, ...lines };
  fs.writeFileSync(`${localizePath}app_${locale}.arb`, JSON.stringify(ja));
};

export const createTypeScriptL10nFiles = async (localizePath: string, values: string[][]) => {
  // ヘッダー行を取得
  const header = values[0];
  if (!header) return;

  // ロケールのインデックスを取得 (key, description の後から)
  const locales = header.slice(2);

  // データ行を取得 (ヘッダー以降)
  const dataRows = values.slice(1);

  // translation.ts の型定義を生成
  const types = dataRows.map((line: string[]) => {
    const [key, description, ...translations] = line;
    const firstTranslation = translations[0] || "";
    return `
  /**
   * ${firstTranslation.replace(/\s/g, "")}: ${description}
   */
  ${key}: string;`;
  });

  fs.writeFileSync(
    `${localizePath}translation.ts`,
    `export interface Translation {${types.join("\n")}\n}`
  );

  // translateFunction.ts のヘルパー関数を生成
  const functions = dataRows
    .map((line: string[]) => {
      const [key, description, ...translations] = line;
      const firstTranslation = translations[0] || "";
      const func = createInputParameterFunction(key, firstTranslation);
      if (!func) return undefined;
      return `
/**
 * ${firstTranslation.replace(/\s/g, "")}: ${description}
 */
${func}`;
    })
    .filter((f: string | undefined) => f !== undefined);

  const functionTemplate = `import { Translation } from "./translation";
${functions.join("\n")}
`;
  fs.writeFileSync(`${localizePath}translateFunction.ts`, functionTemplate);

  // 各言語のファイルを生成
  const template = `import { Translation } from "./translation";\n\nexport const translation: Translation = `;

  locales.forEach((locale, index) => {
    const lines = dataRows
      .map((line: string[]) => {
        const [key, _, ...translations] = line;
        const value: any = {};
        value[key] = translations[index] || "";
        return value;
      })
      .reduce((previousValue: any, currentValue: any) => {
        return { ...previousValue, ...currentValue };
      }, {});

    fs.writeFileSync(`${localizePath}${locale}.ts`, `${template}${JSON.stringify(lines, null, 2)};`);
  });
};
