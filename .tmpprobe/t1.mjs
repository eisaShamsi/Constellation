import { parseDocument, Parser, CST, stringify } from 'yaml';
const cases = {
  dupkey: "tags:\n  - a\ntags:\n  - b\n",
  tabIndent: "title: X\nfoo:\n\t- a\n",
  colonUnquoted: "title: Foo: Bar\n",
  atSign: "note: @home\n",
  tabValue: "a: 1\nb:\tc\n",
  wikilinkBare: "supports:\n  - [[Some Note]]\n",
  blockScalar: "desc: |\n  line1\n  line2\nother: x\n",
  nestedMap: "meta:\n  a: 1\n  b: 2\ntitle: T\n",
};
for (const [k,v] of Object.entries(cases)) {
  const d = parseDocument(v);
  console.log(k, '=> errors:', d.errors.map(e=>e.code).join(','), '| warnings:', d.warnings.map(e=>e.code).join(','));
}
