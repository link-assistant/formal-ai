// Stateless physical line splitting, preserving the existing public Rust-compatible API.
export function rustLines(text) {
  const lines = text.split('\n');
  const terminated = lines.length > 0 && lines[lines.length - 1] === '';
  if (terminated) lines.pop();
  return lines.map((line, index) => (line.endsWith('\r') && (terminated || index < lines.length - 1)
    ? line.slice(0, -1) : line));
}

// JavaScript input adapter for Rust's scalar-valid str/String domain.
// ES string iteration combines surrogate pairs and exposes lone code units.
export function wellFormedText(value) {
  if(typeof value!=='string')return false;
  for(const character of value){
    const point=character.codePointAt(0);
    if(point>=0xd800&&point<=0xdfff)return false;
  }
  return true;
}
