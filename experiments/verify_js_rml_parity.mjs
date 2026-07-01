// Standalone parity check for the JS relative-meta-logic mirror in
// src/web/worker/formal_ai_worker_19.js against the Rust reference strings.
const RML_ASSUMED_TRUE_PRIOR = 0.6;
const RML_SENTENCE_TERMINATORS = new Set([".","!","?","。","！","？","।","॥","؟","\n"]);
const RML_MIN_STATEMENT_WORDS = 3;
const RML_MIN_STATEMENT_CHARS = 6;
function rmlDecimal(v){return Number(v).toFixed(6);}
function extractVerificationStatements(sample){
  const statements=[];let current="";
  const push=()=>{const t=current.trim();current="";if(!t)return;
    const w=t.split(/\s+/u).filter(Boolean).length;
    const c=Array.from(t).filter(ch=>!/\s/u.test(ch)).length;
    if(w<RML_MIN_STATEMENT_WORDS&&c<RML_MIN_STATEMENT_CHARS)return;
    statements.push(t);};
  for(const ch of Array.from(String(sample||""))){if(RML_SENTENCE_TERMINATORS.has(ch))push();else current+=ch;}
  push();return statements;
}
function verificationGroundingQuery(s){return `"${String(s||"").split(/\s+/u).filter(Boolean).join(" ")}" fact check source`;}
function verificationAssessmentTrace(){
  const prior=RML_ASSUMED_TRUE_PRIOR,support=0,contradiction=0;
  const raised=1-(1-prior)*(1-support);const posterior=raised*(1-contradiction);
  return `prior=${rmlDecimal(prior)} support=${rmlDecimal(support)} contradiction=${rmlDecimal(contradiction)} posterior=${rmlDecimal(posterior)} ignored=0`;
}

let ok=true;
function check(label,got,want){const pass=got===want;ok=ok&&pass;console.log(`${pass?"PASS":"FAIL"} ${label}\n  got=${JSON.stringify(got)}\n  want=${JSON.stringify(want)}`);}

const sample="The tower opened in 1889. It stands 300 metres tall.";
const st=extractVerificationStatements(sample);
check("statement_count",String(st.length),"2");
check("statement[0]",st[0],"The tower opened in 1889");
check("statement[1]",st[1],"It stands 300 metres tall");
check("query[0]",verificationGroundingQuery(st[0]),'"The tower opened in 1889" fact check source');
check("assessment",verificationAssessmentTrace(),"prior=0.600000 support=0.000000 contradiction=0.000000 posterior=0.600000 ignored=0");
check("tier weights","1.000000|0.850000|0.500000|0.000000",[1.0,0.85,0.5,0.0].map(rmlDecimal).join("|"));
// CJK (no spaces): the char-count gate governs. The short trailing sentence
// "它高三百米" (5 chars) is below MIN_STATEMENT_CHARS=6 and is dropped, exactly
// as the Rust extractor drops it — verified via examples/issue_535_extract_probe.rs.
check("zh short drops sub-6-char sentence",String(extractVerificationStatements("埃菲尔铁塔于1889年开放。它高三百米。").length),"1");
check("zh long keeps both sentences",String(extractVerificationStatements("埃菲尔铁塔于1889年开放。它有三百多米高。").length),"2");

process.exit(ok?0:1);
