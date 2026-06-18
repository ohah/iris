let checksum = 0;
let sin = Math.sin;
let sqrt = Math.sqrt;

for (let index = 0; index < 600000; index = index + 1) {
  if (sin === sin) {
    checksum = checksum + 1;
  }
  if (sqrt === sqrt) {
    checksum = checksum + 1;
  }
}

globalThis.__irisStrictHbcChecksum = checksum;
