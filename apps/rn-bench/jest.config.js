module.exports = {
  preset: "@react-native/jest-preset",
  transformIgnorePatterns: [
    "node_modules/(?!((jest-)?react-native|@react-native(-community)?|react-native-safe-area-context)/|\\.bun/((jest-)?react-native@|@react-native\\+[^/]+@|@react-native-community\\+[^/]+@|react-native-safe-area-context@))",
  ],
};
