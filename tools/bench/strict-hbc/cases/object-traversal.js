var rows = [];

for (var index = 0; index < 12000; index = index + 1) {
  var nested = {};
  nested.active = index % 5 === 0;
  nested.score = (index * 17) % 1024;

  var row = {};
  row.id = index;
  row.lane = index % 9;
  row.nested = nested;
  rows[index] = row;
}

var checksum = 0;

for (var rowIndex = 0; rowIndex < rows.length; rowIndex = rowIndex + 1) {
  var current = rows[rowIndex];

  if (!current.nested.active) {
    checksum = checksum + current.lane;
  } else {
    checksum = checksum + current.id + current.nested.score;
  }
}

globalThis.__irisStrictHbcChecksum = checksum;
