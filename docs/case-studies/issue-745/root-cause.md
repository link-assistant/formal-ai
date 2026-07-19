# Root cause

The general agentic router mixed two models. Web fetch/search and file writes already used seed-backed semantic roles, while local reads and directory listings still depended on short English arrays. The shell-intent matcher also selected the first matching intent, allowing a generic cue such as “current directory” to beat the more specific request to list that directory.

Routing precedence then compounded the vocabulary gaps. Broad web research ran before typed URL fetching, so “tell me about URL” searched for the literal URL. The write parser accepted marker-led and destination-led requests but not the common assignment frame “set the contents of FILE to VALUE”; that frame fell through to the file-read cue “contents”.

The fix makes file-read actions seed-backed, expands the four-language action/prefix vocabularies, selects the longest shell-intent cue, and adds a safe remainder argument for code search. Typed URL fetch now precedes broad research, and the write parser recognizes assignment frames only when a seeded write action, safe file target, and non-empty value are all present.
