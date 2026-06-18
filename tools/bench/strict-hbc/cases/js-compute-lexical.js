let checksum = 0;

for (let index = 0; index < 600000; index = index + 1) {
  checksum = checksum + Math.sqrt((index % 1000) + 1) * Math.sin(index);
}

checksum = Math.round(checksum * 1000) / 1000;
globalThis.__irisStrictHbcChecksum = checksum;
