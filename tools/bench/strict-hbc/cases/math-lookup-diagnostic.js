var checksum = 0;

for (var index = 0; index < 600000; index = index + 1) {
  if (Math.sin === Math.sin) {
    checksum = checksum + 1;
  }
  if (Math.sqrt === Math.sqrt) {
    checksum = checksum + 1;
  }
}

globalThis.__irisStrictHbcChecksum = checksum;
