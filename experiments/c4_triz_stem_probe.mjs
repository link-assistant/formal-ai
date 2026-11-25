// Mirrors the Rust stem-cue word skip in triz_demands (issue 1637, c4).
const cues = ["противоречи","парадокс","triz","триз","изобретател","contradiction","paradox","trade-off","tradeoff","contradicción","paradoja","विरोधाभास","अंतर्विरोध","矛盾","悖论"];
const cue = new RegExp(cues.map(c=>c.replace(/[.*+?^${}()|[\]\\]/g,"\\$&")).join("|"),"iu");
const spaced = (c) => !/[぀-ヿ㐀-䶿一-鿿豈-﫿가-힯]/u.test(c) && (/[\p{Alphabetic}\p{N}]/u.test(c) || /[ऀ-ॿ]/u.test(c));
for (const p of process.argv.slice(2)) {
  const m = cue.exec(p); let rest = p.slice(m.index+m[0].length);
  if (spaced(m[0].at(-1))) { let i=0; const cs=[...rest]; while(i<cs.length&&spaced(cs[i])) i++; rest=cs.slice(i).join(""); }
  console.log(JSON.stringify(rest.trimStart()));
}
