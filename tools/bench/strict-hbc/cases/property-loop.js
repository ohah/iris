var row = {};
row.id = 0;
row.score = 1;

var checksum = 0;

for (var index = 0; index < 80000; index = index + 1) {
  row.id = index;
  row.score = (row.score + row.id) % 1009;
  checksum = checksum + row.score;
}

globalThis.__irisStrictHbcChecksum = checksum;
