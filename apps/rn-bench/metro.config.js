const path = require("node:path");
const { getDefaultConfig, mergeConfig } = require("@react-native/metro-config");

/**
 * Metro configuration
 * https://reactnative.dev/docs/metro
 *
 * @type {import('@react-native/metro-config').MetroConfig}
 */
const workspaceRoot = path.resolve(__dirname, "../..");

const config = {
  projectRoot: __dirname,
  resolver: {
    nodeModulesPaths: [
      path.resolve(__dirname, "node_modules"),
      path.resolve(workspaceRoot, "node_modules"),
    ],
  },
  watchFolders: [workspaceRoot],
};

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
