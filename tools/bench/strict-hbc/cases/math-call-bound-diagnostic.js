let checksum = 0;
let sin = Math.sin;
let sqrt = Math.sqrt;
let round = Math.round;

for (let index = 0; index < 600000; index = index + 1) {
  checksum = checksum + sqrt((index % 1000) + 1) * sin(index);
}

checksum = round(checksum * 1000) / 1000;
globalThis.__irisStrictHbcChecksum = checksum;
