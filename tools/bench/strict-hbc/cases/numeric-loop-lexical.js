let checksum = 0;

for (let index = 0; index < 200000; index = index + 1) {
  checksum = checksum + (index % 97) * 13 - (index % 11);
}

globalThis.__irisStrictHbcChecksum = checksum;
