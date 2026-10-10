import {

lex}

 from '../self-translation/lexer.mjs';

export function nativeStringContract(source) {

  const tokens=lex(source,

'Rust').filter(t=>t.type!=='comment');

  for (const [at,

token] of tokens.entries()) {

    if(token.type!=='string') continue;

    let prefix=at-1;

while(tokens[prefix]?.text==='#')prefix--;

if(['r',

'b',

'br'].includes(tokens[prefix]?.text)&&tokens[prefix]?.end===tokens[prefix+1]?.start)throw Error('unknown native prefixed string');

    const text=token.text;

    for(let i=1;

i<text.length-1;

i++) {

      const code=text.codePointAt(i);

if(code>=0xd800&&code<=0xdfff)throw Error('native string is not a Unicode scalar sequence');

if(code>0xffff)i++;

      if(text[i]!=='\\')continue;

      const escape=text[++i];

if(['n',

'r',

't',

'0',

'\\',

'"',

"'"].includes(escape))continue;

      if(escape!=='u'||text[++i]!=='{')throw Error('unsupported native string escape');

      const end=text.indexOf('}',

i+1),

digits=text.slice(i+1,

end);

if(end<0||!/^[0-9a-fA-F]{1,6}$/.test(digits))throw Error('unsupported native Unicode escape');

      const scalar=parseInt(digits,

16);

if(scalar>0x10ffff||(scalar>=0xd800&&scalar<=0xdfff))throw Error('invalid native Unicode scalar');

i=end;

    }

  }

}
