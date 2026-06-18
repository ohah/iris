var index = 0;
var checksum = 0;

while (index < 600000) {
  checksum = (checksum + index) % 1000000007;
  index = index + 1;
}

globalThis.__irisStrictHbcChecksum = checksum;
