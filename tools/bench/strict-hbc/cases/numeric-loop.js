var checksum = 0;

for (var index = 0; index < 200000; index = index + 1) {
  checksum = checksum + (index % 97) * 13 - (index % 11);
}

globalThis.__irisStrictHbcChecksum = checksum;
