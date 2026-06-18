let source = new Uint8Array(1000000);

for (let index = 0; index < source.length; index = index + 1) {
  source[index] = index % 251;
}

let copy = new Uint8Array(source.length);
copy.set(source);

let checksum = 0;
for (let sample = 0; sample < copy.length; sample = sample + 10000) {
  checksum = checksum + copy[sample];
}

globalThis.__irisStrictHbcChecksum = checksum;
