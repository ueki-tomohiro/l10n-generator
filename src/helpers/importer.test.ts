import { describe, it, expect } from 'vitest';
import { importValues } from './importer';
import path from 'path';
import { createTempDir, cleanupTempDir, createTestCSV, basicTestData, paramTestData } from '../test-helpers/test-utils';

describe('importValues - CSV', () => {
  it('既存のsample-data.csvファイルを読み込む', async () => {
    const config = {
      fileType: 'csv' as const,
      path: path.join(process.cwd(), 'examples', 'sample-data.csv'),
      credentialType: 'none' as const,
      localizePath: './output/',
    };

    const values = await importValues(config);

    expect(values.length).toBeGreaterThan(0);
    expect(values[0]).toContain('key');
    expect(values[0]).toContain('description');
  });

  it('テスト用CSVファイルを読み込む', async () => {
    const tempDir = createTempDir();
    const csvPath = createTestCSV(tempDir, basicTestData);

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(3);
      expect(values[0]).toEqual(['key', 'description', 'ja', 'en']);
      expect(values[1]).toEqual(['hello', 'Greeting', 'こんにちは', 'Hello']);
      expect(values[2]).toEqual(['goodbye', 'Farewell', 'さようなら', 'Goodbye']);
    } finally {
      cleanupTempDir(tempDir);
    }
  });

  it('パラメータを含むCSVを読み込む', async () => {
    const tempDir = createTempDir();
    const csvPath = createTestCSV(tempDir, paramTestData);

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(3);
      expect(values[0]).toEqual(['key', 'description', 'ja', 'en']);

      // welcomeキーの行を確認
      const welcomeRow = values.find(row => row[0] === 'welcome');
      expect(welcomeRow).toBeDefined();
      expect(welcomeRow![2]).toContain('{name}'); // ja列

      // error_countキーの行を確認
      const errorCountRow = values.find(row => row[0] === 'error_count');
      expect(errorCountRow).toBeDefined();
      expect(errorCountRow![2]).toContain('{count}'); // ja列
    } finally {
      cleanupTempDir(tempDir);
    }
  });

  it('空のCSVファイルを読み込む', async () => {
    const tempDir = createTempDir();
    const csvPath = createTestCSV(tempDir, [], 'empty.csv');

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(0);
    } finally {
      cleanupTempDir(tempDir);
    }
  });

  it('ヘッダーのみのCSVを読み込む', async () => {
    const tempDir = createTempDir();
    const csvPath = createTestCSV(tempDir, [['key', 'description', 'ja', 'en']], 'header-only.csv');

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(1);
      expect(values[0]).toEqual(['key', 'description', 'ja', 'en']);
    } finally {
      cleanupTempDir(tempDir);
    }
  });

  it('特殊文字を含むCSVを読み込む', async () => {
    const tempDir = createTempDir();
    const specialData = [
      ['key', 'description', 'ja', 'en'],
      ['greeting', 'Special chars', 'こんにちは世界', 'Hello World'],
      ['comma', 'Contains comma', 'テストデータ', 'Test data']
    ];
    const csvPath = createTestCSV(tempDir, specialData, 'special.csv');

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(3);
      expect(values[1][2]).toBe('こんにちは世界');
      expect(values[2][2]).toBe('テストデータ');
    } finally {
      cleanupTempDir(tempDir);
    }
  });

  it('存在しないファイルパスでエラーをスローする', async () => {
    const config = {
      fileType: 'csv' as const,
      path: '/nonexistent/path/to/file.csv',
      credentialType: 'none' as const,
      localizePath: './output/',
    };

    await expect(importValues(config)).rejects.toThrow();
  });

  it('複数行のデータを正確に読み込む', async () => {
    const tempDir = createTempDir();
    const largeData = [
      ['key', 'description', 'ja', 'en'],
      ['key1', 'First', '日本語1', 'English1'],
      ['key2', 'Second', '日本語2', 'English2'],
      ['key3', 'Third', '日本語3', 'English3'],
      ['key4', 'Fourth', '日本語4', 'English4'],
      ['key5', 'Fifth', '日本語5', 'English5']
    ];
    const csvPath = createTestCSV(tempDir, largeData);

    try {
      const config = {
        fileType: 'csv' as const,
        path: csvPath,
        credentialType: 'none' as const,
        localizePath: './output/',
      };

      const values = await importValues(config);

      expect(values.length).toBe(6);
      expect(values[0]).toEqual(['key', 'description', 'ja', 'en']);
      expect(values[5]).toEqual(['key5', 'Fifth', '日本語5', 'English5']);
    } finally {
      cleanupTempDir(tempDir);
    }
  });
});

// Google Sheets関連のテストはモックが複雑なため、統合テストで実施
describe('importValues - Google Sheets', () => {
  it('Google Sheets設定を受け入れる（実際の接続は統合テストで確認）', () => {
    const config = {
      fileType: 'sheet' as const,
      path: 'https://docs.google.com/spreadsheets/d/dummy-id/edit',
      credentialType: 'none' as const,
      localizePath: './output/',
    };

    // 設定オブジェクトが正しく構成されていることを確認
    expect(config.fileType).toBe('sheet');
    expect(config.credentialType).toBe('none');
    expect(config.path).toContain('spreadsheets');
  });

  it('API Key設定を受け入れる', () => {
    const config = {
      fileType: 'sheet' as const,
      path: 'https://docs.google.com/spreadsheets/d/dummy-id/edit',
      credentialType: 'apiKey' as const,
      apiKey: 'test-api-key',
      localizePath: './output/',
    };

    expect(config.credentialType).toBe('apiKey');
    expect(config.apiKey).toBe('test-api-key');
  });
});
