# Issue 228 Online Research

Collected on 2026-05-22.

## Domain Evidence

- GameWith, "Genshin Impact | Off-Field Character List":
  <https://gamewith.net/genshin-impact/article/show/56136>
  - The page was available during research and showed "Last Updated:
    May 20, 2026".
  - It presents an "Off-Field / Sub-DPS Character List" and explains
    that the guide covers off-field characters, off-field supports,
    sub-DPS, and the role of off-field characters.
  - This demonstrates that the reported prompt asks for a current
    enumerable web fact, not a fixed local symbolic rule.

- Genshin Impact Wiki / Fandom, "Character Role":
  <https://genshin-impact.fandom.com/wiki/Character_Role>
  - The page defines an Off-Field role and exposes a "Characters by
    Role" table with On-Field, Off-Field, DPS, Support, and
    Survivability columns.
  - This is structured enough for future extractor work, but the
    current product already has a web-search fusion layer that can
    surface this source without adding a Genshin-specific parser.

- Game8, "List of Sub DPS Characters and Rankings":
  <https://game8.co/games/Genshin-Impact/archives/377447>
  - The page was available during research and showed "Last updated on:
    April 11, 2026".
  - It describes the list as a way to find the best off-field DPS
    characters, includes a Sub DPS character table, and gives examples
    of off-field damage/application strengths for characters such as
    Furina, Fischl, Yelan, Xingqiu, Nahida, Xiangling, Albedo, Yae Miko,
    Rosaria, Thoma, Ganyu, Collei, Kokomi, Beidou, and Kaeya.
  - This confirms that the answer set changes with game updates and is
    better treated as live research.

## Component Evidence

- Formal-ai already has a deterministic browser web-search planner:
  DuckDuckGo Instant Answer, Internet Archive, Wikipedia REST,
  Wikidata, and Wiktionary are queried and combined with Reciprocal
  Rank Fusion.
- Cormack, Clarke, and Buettcher introduced Reciprocal Rank Fusion:
  <https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf>
  The existing implementation uses `k = 60` and records traceable
  `web_search:*` evidence lines.

## Conclusion

The issue should not be fixed by seeding a static list of Genshin
characters. The durable fix is to route enumeration-style research
requests, such as "list all X with Y", into the existing deterministic
web-search path so the live browser worker can gather current,
source-linked results without neural inference.
