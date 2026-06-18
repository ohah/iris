var payload = [];

for (var index = 0; index < 8000; index = index + 1) {
  var meta = {};
  meta.lane = index % 7;
  meta.label = "group";

  var points = [];
  points[0] = index;
  points[1] = index * 2;
  points[2] = index * 3;

  var row = {};
  row.active = index % 3 === 0;
  row.id = index;
  row.meta = meta;
  row.name = "item";
  row.points = points;

  payload[index] = row;
}

var encoded = JSON.stringify(payload);
var decoded = JSON.parse(encoded);
var checksum = 0;

for (var rowIndex = 0; rowIndex < decoded.length; rowIndex = rowIndex + 1) {
  var current = decoded[rowIndex];
  checksum = checksum + current.id + current.points[2];
}

globalThis.__irisStrictHbcChecksum = checksum;
