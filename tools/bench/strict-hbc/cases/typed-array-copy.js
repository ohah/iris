var source = new Uint8Array(1000000);

for (var index = 0; index < source.length; index = index + 1) {
  source[index] = index % 251;
}

var copy = new Uint8Array(source.length);
copy.set(source);

var checksum = 0;
for (var sample = 0; sample < copy.length; sample = sample + 10000) {
  checksum = checksum + copy[sample];
}

globalThis.__irisStrictHbcChecksum = checksum;
