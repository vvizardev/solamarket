/** @type {import('jest').Config} */
module.exports = {
  preset: "ts-jest",
  testEnvironment: "node",
  roots: ["<rootDir>"],
  testMatch: ["**/*.test.ts"],
  transform: {
    "^.+\\.tsx?$": [
      "ts-jest",
      {
        tsconfig: {
          strict: true,
          esModuleInterop: true,
          target: "ES2020",
          module: "CommonJS",
        },
      },
    ],
  },
  moduleNameMapper: {
    "^@solana/web3.js$": require.resolve("@solana/web3.js"),
  },
};
