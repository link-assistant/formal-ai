# Issue 649 — Online Research (summarized and cited)

Per [NON-GOALS.md](../../../../NON-GOALS.md) (*"Research notes should not copy
large external texts; they should summarize and cite sources"*), every claim
below is paraphrased and linked to its source; only short definitional phrases
are quoted.

---

## 1. The source the issue is "inspired by"

The issue opens with *"Inspired by <https://youtu.be/kYkIdXwW2AE>"*. That video is
**"Yann LeCun's $1B Bet Against LLMs [Part 1]"** by **Welch Labs**
([youtube.com/watch?v=kYkIdXwW2AE](https://www.youtube.com/watch?v=kYkIdXwW2AE);
[Part 2](https://www.youtube.com/watch?v=v_jDvpEGTIg)). It explains Yann LeCun's
thesis that autoregressive LLMs will not reach general intelligence and that the
path forward is **world models** — non-generative predictors that learn an
internal representation of the environment and predict its *next state* given an
action, enabling planning and cause-effect reasoning rather than next-token
prediction (Welch Labs; summarized by
[Geeky Gadgets](https://www.geeky-gadgets.com/lecun-billion-dollar-ai-bet/) and
[optim.vc](https://www.optim.vc/catching-up-on-yann-lecun-jepa-world-models-ami-labs-and-the-war-against-llms/)).

LeCun's own one-line definition of a world model — the phrase the issue title
paraphrases — is *"an abstract digital twin of reality that an AI can use to
understand the world, **predict the consequences of its actions**, and plan
accordingly"* ([Meta AI, I-JEPA announcement](https://ai.meta.com/blog/yann-lecun-ai-model-i-jepa/);
[Taskade, *AI World Models*](https://www.taskade.com/blog/ai-world-models)). The
issue's title — *"Predicting consequences of actions using world models/formal
systems/contexts"* — is this idea, re-cast onto the repository's **symbolic**
substrate instead of a learned embedding space.

### The key divergence from the video

The video's world models (JEPA / V-JEPA 2, energy-based planning) represent state
as a **learned embedding** and predict in that latent space
([Meta I-JEPA](https://ai.meta.com/blog/yann-lecun-ai-model-i-jepa/);
[Wikipedia, *World model (AI)*](https://en.wikipedia.org/wiki/World_model_(artificial_intelligence))).
The issue explicitly rejects that substrate: *"Instead of embeddings as a meaning
representation, we can use the links networks as the meaning representation."*
So `formal-ai` keeps LeCun's **goal** (predict consequences, plan) but swaps the
**representation** from opaque vectors to an inspectable links network — which is
exactly the symbolic-AI lineage the repo already documents
([issue-451 case study](../../issue-451/README.md)).

---

## 2. Relative Meta-Logic (the mechanism the issue mandates)

The issue requires *"relative dependent logic (relative meta logic)
<https://github.com/link-foundation/relative-meta-logic>"* and that *"if we change
something in the world model or formal system or context, all statements
probabilities are recalculated."*

**Relative Meta-Logic (RML)**, formerly *Associative-Dependent Logic*
([link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic)),
is a probabilistic, many-valued logic built on **Links Notation (LiNo)** that
"can reason about anything **relative to** given probability of input statements."
Salient features for this issue:

- **Configurable valence / range**: Boolean (2), Kleene ternary (3), N-valued, or
  continuous `[0,1]` / balanced `[-1,1]`; redefinable truth constants
  (`true`/`false`/`unknown`/`undefined`).
- **Statements are links**; probabilities are assertions over them
  (`((a = b) has probability 0.8)`); queries `(? expr)` evaluate **at query
  time**, so *changing any input probability or operator immediately changes every
  dependent query result* — this is the "recalculate on change" behaviour the
  issue asks for, realized by re-evaluation rather than by cached truth.
- **Redefinable operators / aggregators**: `min`, `max`, `avg`, `product`,
  `probabilistic_sum`; **Belnap** `both…and` / `neither…nor` treat contradiction
  and gaps as first-class.
- **Decimal-precision arithmetic** for reproducible truth values (e.g.
  `0.1 + 0.2 = 0.3` exactly), and dual JS/Rust implementations that produce
  identical results.

The repository already ports the core of this into
[`src/relative_meta_logic.rs`](../../../../src/relative_meta_logic.rs) (bounded
`TruthValue`, the same aggregator family, source-trust weighting, decimal grid)
— so RML is *present*, and the issue's ask is to lift it from per-statement
assessment to **per-context, dependency-aware recalculation**.

---

## 3. Classical prior art for "current state → action → target state"

The issue's structure — a **current** world state, a **target** world state, the
**difference** between them, and **predicting the consequences of actions** — is
the textbook shape of **automated planning**, which the field has formalized for
50 years.

### STRIPS / PDDL (state + goal + action effects)
STRIPS (Stanford Research Institute Problem Solver) models a problem as a
**discrete, deterministic** world: an *initial state* (a set of true atoms), a
*goal* (a boolean condition over atoms), and *actions* whose **effects** split
into an **add list** (atoms made true) and a **delete list** (atoms made false)
([CS540 planning notes, Wisconsin](https://pages.cs.wisc.edu/~dyer/cs540/notes/planning.html);
[Edinburgh PDDL notes](http://www.inf.ed.ac.uk/teaching/courses/propm/papers/pddl.html)).
PDDL generalizes STRIPS with typed predicates and richer goals
([Helmert, *Concise finite-domain representations for PDDL*](https://www.sciencedirect.com/science/article/pii/S0004370208001926/pdf)).
This is the exact primitive the issue needs: **the "difference from the current
state" is the goal-minus-state delta, and "predicting consequences of an action"
is applying that action's add/delete effects to the current state.**

### Situation calculus
The situation calculus represents changing worlds as a sequence of *situations*
produced by actions, with *fluents* whose truth depends on the situation. It is
*"representation strong, but reasoning weak"* because naive resolution over
fine-grained facts is slow ([Aalto AI-planning overview](https://users.aalto.fi/~rintanj1/planning.html)).
Its lesson for `formal-ai`: keep the **representation** (states as link sets,
actions as transforms) but delegate heavy reasoning to bounded, deterministic
procedures rather than open-ended resolution — which is already the repo's
`proof_engine` discipline.

### Probabilistic planning
When actions have uncertain effects, planning becomes **probabilistic** (MDP /
weighted-model-counting style), so each successor state carries a probability
([*Probabilistic Planning via Heuristic Forward Search and Weighted Model
Counting*, arXiv 1111.0044](https://arxiv.org/pdf/1111.0044)). This is the bridge
to RML: an action's consequence is not a single next state but a **distribution
over dependent statements**, recomputed by RML aggregators.

---

## 4. Classical prior art for "recalculate all statements when the world changes"

The issue's recalculation requirement is precisely what **Truth Maintenance
Systems (TMS)** were invented for.

- **JTMS (Justification-based TMS)** records a *justification* for every belief;
  when a fact is added or retracted it **re-evaluates the justifications of all
  affected beliefs**, keeping the belief set consistent
  ([Number Analytics, *Mastering TMS*](https://www.numberanalytics.com/blog/mastering-truth-maintenance-systems);
  [Buffalo, *Belief Revision and TMS overview*](https://cse.buffalo.edu/~shapiro/Papers/br-overview.pdf)).
- **ATMS (Assumption-based TMS)** labels each belief with the **set of
  assumptions** that support it, so it can *"maintain multiple contexts of belief
  simultaneously"* — whereas JTMS holds only one context at a time
  ([Number Analytics](https://www.numberanalytics.com/blog/mastering-truth-maintenance-systems)).
  This is the **direct classical analog of the issue's "merge / split world
  models (contexts)"**: an ATMS *context* is a consistent set of assumptions, and
  merging/splitting contexts is switching or combining assumption sets.
- **AGM belief revision** (Alchourrón–Gärdenfors–Makinson) gives the rationality
  postulates for *expansion / revision / contraction* of a belief set; the ATMS
  can be simulated inside AGM by encoding justifications as *epistemic
  entrenchment*, and AGM expansion/contraction implements ATMS context switches
  ([Dixon & Foo, *Connections Between the ATMS and AGM Belief Revision*, IJCAI-93](https://www.ijcai.org/Proceedings/93-1/Papers/075.pdf)).

**Takeaway:** the issue's "each context has dependent statements; changing the
world recalculates all probabilities" is TMS **generalized from Boolean beliefs
to RML truth values**, and "merge / split contexts" is ATMS **context
combination generalized from assumption sets to link-network world models**. The
repo does not need to invent these semantics — it needs to *re-express* the
JTMS/ATMS dependency graph over its links network and drive recalculation with
`relative_meta_logic`.

---

## 5. Where the neural world-model literature agrees and disagrees

The 2024–2026 world-model literature ([Wikipedia, *World model (AI)*](https://en.wikipedia.org/wiki/World_model_(artificial_intelligence));
[Fei-Fei Li, *A Functional Taxonomy of World Models*](https://drfeifei.substack.com/p/a-functional-taxonomy-of-world-models);
[Introl, *World Models Race 2026*](https://introl.com/blog/world-models-race-agi-2026))
converges on world models as systems that *represent state, model dynamics, and
predict next state under actions*. `formal-ai` shares the **functional
decomposition** (state + dynamics + prediction) but takes the opposite stance on
**representation**: the state is a human-readable links network and the dynamics
are explicit, inspectable rules — trading the neural models' perceptual coverage
for determinism, provenance, and the ability to *explain the current and target
state exactly*, which the issue makes a first-class requirement. Nothing in the
literature contradicts a symbolic world model for the discrete, dialogue-scoped
tasks the repo targets; it simply occupies the explainable end of Fei-Fei Li's
taxonomy.

---

## 6. Source list

| # | Source | Used for |
|---|---|---|
| 1 | [Welch Labs — *Yann LeCun's $1B Bet Against LLMs* (Part 1)](https://www.youtube.com/watch?v=kYkIdXwW2AE), [Part 2](https://www.youtube.com/watch?v=v_jDvpEGTIg) | The video the issue is inspired by |
| 2 | [Meta AI — I-JEPA announcement](https://ai.meta.com/blog/yann-lecun-ai-model-i-jepa/) | LeCun's "predict the consequences of its actions" world-model definition |
| 3 | [Taskade — *AI World Models: History, JEPA & Inference Scaling*](https://www.taskade.com/blog/ai-world-models) | World-model / JEPA background |
| 4 | [Wikipedia — *World model (artificial intelligence)*](https://en.wikipedia.org/wiki/World_model_(artificial_intelligence)) | Definition; state+dynamics+prediction framing |
| 5 | [Fei-Fei Li — *A Functional Taxonomy of World Models*](https://drfeifei.substack.com/p/a-functional-taxonomy-of-world-models) | Functional taxonomy placement |
| 6 | [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic) | RML valence/aggregators/query-time recalculation |
| 7 | [Wisconsin CS540 — Planning notes](https://pages.cs.wisc.edu/~dyer/cs540/notes/planning.html) · [Edinburgh PDDL notes](http://www.inf.ed.ac.uk/teaching/courses/propm/papers/pddl.html) | STRIPS/PDDL state, goal, add/delete effects |
| 8 | [Helmert — *Concise finite-domain representations for PDDL*](https://www.sciencedirect.com/science/article/pii/S0004370208001926/pdf) | PDDL grounding |
| 9 | [Aalto — *A brief overview of AI planning*](https://users.aalto.fi/~rintanj1/planning.html) | Situation calculus |
| 10 | [arXiv 1111.0044 — *Probabilistic Planning via Heuristic Forward Search and WMC*](https://arxiv.org/pdf/1111.0044) | Probabilistic action effects |
| 11 | [Number Analytics — *Mastering Truth Maintenance Systems*](https://www.numberanalytics.com/blog/mastering-truth-maintenance-systems) | JTMS/ATMS recomputation and multiple contexts |
| 12 | [Shapiro et al. — *Belief Revision and TMS: An Overview*](https://cse.buffalo.edu/~shapiro/Papers/br-overview.pdf) | JTMS/ATMS/AGM relationship |
| 13 | [Dixon & Foo — *Connections Between the ATMS and AGM Belief Revision*, IJCAI-93](https://www.ijcai.org/Proceedings/93-1/Papers/075.pdf) | ATMS context switches as AGM operations |
| 14 | [Introl — *World Models Race 2026*](https://introl.com/blog/world-models-race-agi-2026) | 2026 world-model landscape |
