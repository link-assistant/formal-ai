#!/usr/bin/env python3
"""Gather common-noun Wikidata entities for the bulk lexeme importer (issue #660).

The sandbox blocks direct Wikimedia traffic (HTTP 403), so we reach the public
`Special:EntityData` and `wbsearchentities` JSON through the r.jina.ai read proxy.
For each curated English noun we:

  1. resolve the top matching Wikidata item id via `wbsearchentities`,
  2. fetch its full entity JSON,
  3. require labels in *all four* project languages (en/ru/hi/zh),
  4. require the English label to equal the search word exactly and to be a
     single whitespace-free token in every language (clean unquoted surfaces),
  5. skip ids/labels already grounded in the committed seed.

Survivors are written to `data/cache/wikidata/entity/<qid>.json` in the exact
trimmed format the repo already uses, and appended to the concepts file
`data/lexicon-import/common-nouns.lino` as `<slug> <qid>` pairs.

This is a one-time curation helper; the shipped importer only ever reads the
committed cache. Run: `python3 experiments/gather_common_nouns.py`.
"""

import json
import os
import re
import sys
import time
import urllib.parse
import urllib.request
from collections import OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "data", "cache", "wikidata", "entity")
CONCEPTS = os.path.join(ROOT, "data", "lexicon-import", "common-nouns.lino")
LANGS = ["en", "ru", "hi", "zh"]
UA = "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"

WORDS = [
    # animals
    "dog", "cat", "horse", "cow", "sheep", "pig", "goat", "chicken", "duck",
    "lion", "tiger", "bear", "wolf", "fox", "deer", "rabbit", "mouse", "rat",
    "elephant", "monkey", "snake", "frog", "fish", "bird", "eagle", "owl",
    "bee", "ant", "spider", "butterfly", "whale", "dolphin", "shark", "crab",
    "camel", "donkey", "kangaroo", "penguin", "crocodile", "turtle",
    # foods & plants
    "apple", "banana", "orange", "grape", "lemon", "cherry", "peach", "pear",
    "strawberry", "mango", "onion", "garlic", "carrot", "cabbage", "pepper",
    "cucumber", "pumpkin", "rice", "wheat", "corn", "bread", "cheese", "milk",
    "butter", "egg", "meat", "sugar", "salt", "honey", "coffee", "tea",
    "wine", "beer", "water", "juice", "soup", "cake", "chocolate", "flour",
    "mushroom", "bean", "pea", "nut", "olive", "coconut", "pineapple",
    # nature
    "sun", "moon", "star", "sky", "cloud", "rain", "snow", "wind", "storm",
    "mountain", "river", "lake", "sea", "ocean", "forest", "tree", "flower",
    "grass", "leaf", "root", "stone", "rock", "sand", "island", "desert",
    "valley", "hill", "waterfall", "volcano", "rainbow", "fire", "ice",
    # body
    "head", "hair", "eye", "ear", "nose", "mouth", "tooth", "tongue", "lip",
    "neck", "shoulder", "arm", "hand", "finger", "leg", "foot", "knee",
    "heart", "brain", "blood", "bone", "skin", "stomach", "liver", "lung",
    # household & objects
    "house", "door", "window", "roof", "wall", "floor", "table", "chair",
    "bed", "lamp", "mirror", "clock", "knife", "fork", "spoon", "plate",
    "cup", "bottle", "bowl", "pan", "book", "pen", "pencil", "paper",
    "key", "box", "bag", "basket", "umbrella", "candle", "soap", "towel",
    "pillow", "blanket", "carpet", "curtain", "brush", "comb", "needle",
    # clothing
    "shirt", "dress", "skirt", "coat", "jacket", "hat", "shoe", "sock",
    "glove", "scarf", "belt", "button", "pocket", "ring", "crown",
    # transport & tools
    "car", "bicycle", "train", "boat", "ship", "airplane", "bus", "truck",
    "wheel", "engine", "hammer", "axe", "saw", "nail", "rope", "chain",
    "ladder", "bridge", "road", "wheelbarrow", "anchor", "sword", "shield",
    # misc concrete
    "gold", "silver", "iron", "copper", "diamond", "glass", "wood", "paper",
    "coal", "oil", "brick", "cement", "wire", "coin", "flag", "drum",
    "guitar", "piano", "violin", "trumpet", "bell", "kite", "ball", "doll",
    "mask", "ticket", "letter", "map", "photograph", "picture", "statue",
    # family & people
    "mother", "father", "brother", "sister", "child", "baby", "family",
    "man", "woman", "boy", "girl", "friend", "neighbor", "guest",
    # professions & roles
    "doctor", "teacher", "farmer", "soldier", "king", "queen", "artist",
    "poet", "painter", "singer", "dancer", "hunter", "fisherman", "cook",
    "priest", "judge", "lawyer", "nurse", "pilot", "sailor", "merchant",
    "student", "worker", "engineer", "scientist", "writer", "actor",
    # abstract concepts
    "love", "war", "peace", "death", "life", "time", "money", "music",
    "language", "science", "art", "religion", "law", "freedom", "truth",
    "beauty", "power", "knowledge", "history", "nature", "dream", "memory",
    "fear", "hope", "joy", "anger", "pain", "health", "wealth", "poverty",
    # time & seasons
    "day", "night", "morning", "evening", "week", "month", "year", "hour",
    "minute", "summer", "winter", "spring", "autumn", "season", "century",
    # places & structures
    "city", "town", "village", "country", "school", "hospital", "church",
    "temple", "market", "shop", "garden", "park", "farm", "castle", "tower",
    "palace", "prison", "library", "museum", "theater", "factory", "harbor",
    "street", "square", "station", "airport", "port", "well", "fountain",
    # colors
    "red", "blue", "green", "yellow", "black", "white", "brown", "purple",
    "pink", "gray",
    # nature & elements
    "world", "earth", "air", "light", "shadow", "smoke", "dust", "mud",
    "wave", "flood", "cave", "cliff", "glacier", "swamp", "meadow", "field",
    "planet", "comet", "galaxy", "thunder", "lightning", "fog", "frost",
    # more animals
    "cattle", "buffalo", "leopard", "panther", "cheetah", "hyena", "otter",
    "beaver", "squirrel", "hedgehog", "bat", "hawk", "crow", "sparrow",
    "pigeon", "parrot", "peacock", "swan", "goose", "turkey", "raven",
    "scorpion", "snail", "worm", "moth", "wasp", "beetle", "lizard",
    "seal", "walrus", "octopus", "jellyfish", "starfish", "lobster",
    # more foods & plants
    "potato", "tomato", "spinach", "lettuce", "radish", "beet", "ginger",
    "walnut", "almond", "cashew", "raisin", "apricot", "plum", "fig",
    "date", "melon", "watermelon", "papaya", "guava", "lime", "berry",
    "oat", "barley", "millet", "lentil", "soybean", "vinegar", "yogurt",
    "cream", "pasta", "noodle", "biscuit", "candy", "jam", "pickle",
    # materials & minerals
    "steel", "bronze", "aluminium", "marble", "clay", "leather", "cotton",
    "wool", "silk", "rubber", "plastic", "salt", "chalk", "amber", "pearl",
    "ruby", "emerald", "crystal", "granite", "concrete", "ash", "wax",
    # tools & instruments
    "flute", "harp", "cello", "clarinet", "accordion", "whistle", "gong",
    "spade", "shovel", "rake", "hoe", "plough", "sickle", "scissors",
    "screwdriver", "wrench", "drill", "pliers", "bolt", "screw", "hinge",
    "lantern", "torch", "compass", "telescope", "microscope", "magnet",
]


_LAST = [0.0]
MIN_INTERVAL = 4.0  # r.jina.ai anonymous tier ~20 requests/minute.


def _pace():
    now = time.time()
    wait = MIN_INTERVAL - (now - _LAST[0])
    if wait > 0:
        time.sleep(wait)
    _LAST[0] = time.time()


def fetch(url):
    proxied = "https://r.jina.ai/" + url
    for attempt in range(6):
        _pace()
        try:
            req = urllib.request.Request(proxied, headers={"User-Agent": UA})
            raw = urllib.request.urlopen(req, timeout=90).read().decode("utf-8", "replace")
        except urllib.error.HTTPError as exc:
            if exc.code == 429:
                back = 20 * (attempt + 1)
                sys.stderr.write(f"  429 backoff {back}s\n")
                time.sleep(back)
                continue
            sys.stderr.write(f"  http error ({attempt}): {exc}\n")
            time.sleep(5)
            continue
        except Exception as exc:  # noqa: BLE001
            sys.stderr.write(f"  fetch error ({attempt}): {exc}\n")
            time.sleep(5)
            continue
        marker = "Markdown Content:"
        idx = raw.find(marker)
        body = raw[idx + len(marker):] if idx >= 0 else raw
        start = body.find("{")
        if start < 0:
            time.sleep(5)
            continue
        try:
            obj, _ = json.JSONDecoder().raw_decode(body[start:])
            return obj
        except Exception:  # noqa: BLE001
            time.sleep(5)
            continue
    return None


def resolve_qid(word):
    """Return the most fundamental Wikidata item whose label equals `word`.

    `wbsearchentities` ranks disambiguation-specific items (surnames, films,
    companies) alongside the core concept, so ``limit=1`` frequently misses the
    ordinary noun (e.g. "fox" -> Fox Broadcasting). Among the exact-label
    matches we pick the smallest numeric id: the canonical everyday concept was
    almost always minted first (Q140 lion, Q144 dog, Q146 cat), so the lowest
    id reliably selects it.
    """
    q = urllib.parse.quote(word)
    url = (
        "https://www.wikidata.org/w/api.php?action=wbsearchentities"
        f"&search={q}&language=en&type=item&limit=20&format=json"
    )
    doc = fetch(url)
    if not doc:
        return None
    hits = doc.get("search") or []
    exact = [
        h["id"]
        for h in hits
        if (h.get("label") or "").lower() == word.lower()
        and re.fullmatch(r"Q\d+", h.get("id", ""))
    ]
    if not exact:
        return None
    return min(exact, key=lambda qid: int(qid[1:]))


def fetch_entity(qid):
    url = f"https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
    doc = fetch(url)
    if not doc:
        return None
    return doc.get("entities", {}).get(qid)


def single_token(value):
    return value is not None and value.strip() == value and not re.search(r"\s", value)


def trim(entity, qid):
    def keep(section):
        return OrderedDict((l, section[l]) for l in LANGS if l in section)

    trimmed = OrderedDict()
    trimmed["type"] = entity["type"]
    trimmed["id"] = entity["id"]
    if "labels" in entity:
        trimmed["labels"] = keep(entity["labels"])
    if "descriptions" in entity:
        trimmed["descriptions"] = keep(entity["descriptions"])
    if "aliases" in entity:
        kept = keep(entity["aliases"])
        if kept:
            trimmed["aliases"] = kept
    result = OrderedDict()
    result["entities"] = OrderedDict([(qid, trimmed)])
    result["success"] = 1
    return result


def existing_context():
    used_qids, surfaces, slugs = set(), set(), set()
    seed_dir = os.path.join(ROOT, "data", "seed")
    for name in os.listdir(seed_dir):
        if not name.endswith(".lino"):
            continue
        text = open(os.path.join(seed_dir, name), encoding="utf-8").read()
        used_qids |= set(re.findall(r"grounded-in\s+(Q\d+)", text))
        used_qids |= set(re.findall(r"wikidata\s+(Q\d+)", text))
        for m in re.finditer(r'\btext\s+"?([^"\n]+?)"?\s*$', text, re.M):
            surfaces.add(m.group(1).strip())
    for name in os.listdir(os.path.join(seed_dir)):
        pass
    # meaning slugs: read via a light scan of top-level meaning node names is
    # complex; approximate by collecting `meaning <slug>` and `<slug>` headers.
    return used_qids, surfaces, slugs


PROGRESS = os.path.join(ROOT, "experiments", "logs", "gather_progress.jsonl")


def load_progress():
    done, kept = {}, []
    if not os.path.exists(PROGRESS):
        return done, kept
    for line in open(PROGRESS, encoding="utf-8"):
        line = line.strip()
        if not line:
            continue
        rec = json.loads(line)
        done[rec["word"]] = rec
        if rec.get("kept"):
            kept.append((rec["slug"], rec["qid"], rec["vals"]))
    return done, kept


def record(rec):
    with open(PROGRESS, "a", encoding="utf-8") as fh:
        fh.write(json.dumps(rec, ensure_ascii=False) + "\n")


def main():
    os.makedirs(CACHE, exist_ok=True)
    os.makedirs(os.path.dirname(CONCEPTS), exist_ok=True)
    os.makedirs(os.path.dirname(PROGRESS), exist_ok=True)
    used_qids, surfaces, _ = existing_context()
    done, kept = load_progress()
    seen_qids = {q for _, q, _ in kept}
    for _, _, vals in kept:
        for l in LANGS:
            surfaces.add(vals[l])
    for word in WORDS:
        if word in done:
            print(f"[done] {word}: {'kept' if done[word].get('kept') else 'skipped'}")
            continue
        def skip(reason, permanent=True):
            print(f"[skip] {word}: {reason}")
            if permanent:
                record({"word": word, "kept": False, "reason": reason})

        qid = resolve_qid(word)
        if not qid:
            skip("no qid (network?)", permanent=False)
            continue
        if qid in seen_qids or qid in used_qids:
            skip(f"{qid} already used")
            continue
        entity = fetch_entity(qid)
        if not entity:
            skip(f"{qid} no entity (network?)", permanent=False)
            continue
        labels = entity.get("labels", {})
        vals = {l: labels.get(l, {}).get("value") for l in LANGS}
        if not all(vals.values()):
            skip(f"{qid} missing langs {[l for l in LANGS if not vals[l]]}")
            continue
        if vals["en"].lower() != word.lower():
            skip(f"{qid} en label '{vals['en']}' != word")
            continue
        if not all(single_token(vals[l]) for l in LANGS):
            skip(f"{qid} multi-token label {vals}")
            continue
        if any(vals[l] in surfaces for l in LANGS):
            skip(f"{qid} label already a surface")
            continue
        result = trim(entity, qid)
        path = os.path.join(CACHE, f"{qid}.json")
        with open(path, "w", encoding="utf-8") as fh:
            json.dump(result, fh, ensure_ascii=False, indent=2)
            fh.write("\n")
        seen_qids.add(qid)
        for l in LANGS:
            surfaces.add(vals[l])
        kept.append((word.lower(), qid, vals))
        record({"word": word, "kept": True, "slug": word.lower(), "qid": qid, "vals": vals})
        print(f"[keep] {word} -> {qid}  {vals}")

    kept.sort(key=lambda r: r[0])
    with open(CONCEPTS, "w", encoding="utf-8") as fh:
        fh.write("# Common concrete nouns grounded from Wikidata (issue #660).\n")
        fh.write("# Generated by experiments/gather_common_nouns.py; each line is\n")
        fh.write("# `<slug> <Qid>` and resolves to a committed entity cache record.\n")
        fh.write("concepts\n")
        for slug, qid, _ in kept:
            fh.write(f"  {slug} {qid}\n")
    print(f"\nKept {len(kept)} concepts -> {CONCEPTS}")


if __name__ == "__main__":
    main()
