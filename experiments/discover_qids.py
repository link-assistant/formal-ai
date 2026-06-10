import urllib.parse, json, subprocess, sys
UA="formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
# (slug, search_term, expected_label_token, description_substring_hint)
BATCH=[
 # numbers
 ("zero","0 number","zero","natural number"),
 ("one","1 number","1","natural number"),
 ("two","2 number","2","natural number"),
 ("three","3 number","3","natural number"),
 ("four","4 number","4","natural number"),
 ("five","5 number","5","natural number"),
 ("six","6 number","6","natural number"),
 ("seven","7 number","7","natural number"),
 ("eight","8 number","8","natural number"),
 ("nine","9 number","9","natural number"),
 ("ten","10 number","10","natural number"),
 ("cardinal_number","cardinal numeral","cardinal","number"),
 ("unit","unit of measurement","unit","measurement"),
 # quantities/dimensions
 ("length","length","length","distance"),
 ("mass","mass","mass","property"),
 ("time","time","time",""),
 ("temperature","temperature","temperature","physical quantity"),
 ("data_storage","data storage","data storage",""),
 # units
 ("kilobyte","kilobyte","kilobyte","unit of digital information"),
 ("megabyte","megabyte","megabyte","unit of digital information"),
 ("gigabyte","gigabyte","gigabyte","unit of digital information"),
 ("ton","tonne","tonne","unit of mass"),
 ("fahrenheit","degree Fahrenheit","fahrenheit","temperature"),
 ("kelvin","kelvin","kelvin","temperature"),
 # math
 ("cosine","cosine","cosine","trigonometric"),
 ("tangent","tangent function","tangent","trigonometric"),
 ("modulo","modulo operation","modulo",""),
 ("arithmetic_operation","arithmetic operation","arithmetic",""),
 ("mathematical_function","function mathematics","function","mathematic"),
 # programming languages
 ("program_language_rust","Rust programming language","rust","programming language"),
 ("program_language_python","Python programming language","python","programming language"),
 ("program_language_javascript","JavaScript","javascript","programming language"),
 ("program_language_typescript","TypeScript","typescript","programming language"),
 ("program_language_go","Go programming language","go","programming language"),
 ("program_language_c","C programming language","c","programming language"),
 ("program_language_cpp","C++","c++","programming language"),
 ("program_language_java","Java programming language","java","programming language"),
 ("program_language_csharp","C Sharp programming language","c#","programming language"),
 ("program_language_ruby","Ruby programming language","ruby","programming language"),
 ("program_language","programming language","programming language",""),
 # natural languages
 ("language_english","English language","english","language"),
 ("language_russian","Russian language","russian","language"),
 ("language_hindi","Hindi","hindi","language"),
 ("language_chinese","Chinese language","chinese","language"),
 ("human_language","natural language","natural language",""),
 # concrete nouns
 ("apple","apple fruit","apple","fruit"),
 ("tomato","tomato","tomato",""),
 ("cucumber","cucumber","cucumber",""),
 ("potato","potato","potato",""),
 ("carrot","carrot","carrot",""),
 ("bread","bread","bread","food"),
 ("water","water","water",""),
 # facts predicates
 ("capital","capital city","capital","seat of government"),
 ("population","population","population",""),
 ("continent","continent","continent",""),
 # lexical
 ("noun","noun","noun","part of speech"),
 ("part_of_speech","part of speech","part of speech",""),
 ("noun_phrase","noun phrase","noun phrase",""),
]
def search(term):
    url="https://www.wikidata.org/w/api.php?action=wbsearchentities&search=%s&language=en&format=json&limit=8&type=item"%urllib.parse.quote(term)
    out=subprocess.run(["curl","-sfL","-A",UA,url],capture_output=True,text=True)
    if out.returncode!=0: return []
    return json.loads(out.stdout).get("search",[])
ok=[]; fail=[]
for slug,term,tok,desc in BATCH:
    cands=search(term)
    pick=None
    for c in cands:
        lab=(c.get("label") or "").lower()
        d=(c.get("description") or "").lower()
        if tok.lower() in lab and (desc.lower() in d if desc else True):
            pick=c; break
    if pick:
        ok.append((slug,pick["id"],tok,pick.get("label"),pick.get("description","")[:50]))
    else:
        fail.append((slug,term,tok,[ (c["id"],c.get("label"),c.get("description","")[:40]) for c in cands[:3]]))
print("=== RESOLVED (%d) ==="%len(ok))
for s,q,t,l,d in ok: print(f'("{s}","{q}","{t}"), // {l} :: {d}')
print("\n=== UNRESOLVED (%d) ==="%len(fail))
for s,term,t,c in fail: print(f'{s} (term="{term}" tok="{t}") cands={c}')
