import fs from "fs";
import { parse } from "csv-parse";
import isArray from "lodash/isArray";

type ImportCSV = (filePath: string) => Promise<string[][]>;

export const importCSV: ImportCSV = async (filePath) => {
  const buffer = fs.readFileSync(filePath);

  const parser = parse(buffer);
  const lines: string[][] = [];
  for await (const record of parser) {
    if (isArray(record)) lines.push(record);
  }
  return lines;
};
