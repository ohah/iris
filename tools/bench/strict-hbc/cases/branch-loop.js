var checksum = 0;
var index = 0;
var even = 0;
var odd = 0;

for (index = 0; index < 120000; index = index + 1) {
  if (index % 2 === 0) {
    even = even + (index % 97);
  } else {
    odd = odd + (index % 89);
  }
}

checksum = even - odd;
globalThis.__irisStrictHbcChecksum = checksum;
