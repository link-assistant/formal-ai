# Issue 701 online research: closing the adoption gap

Issue #701 asks a narrow question: what does it take for a learning loop to stop at
*proposal* and instead **adopt** — take a failure at the frontier of current capability,
derive a candidate improvement, validate it, and land it so that later behaviour
demonstrably differs? The neural/LLM agent literature has converged on a small number of
answers to this, and — importantly for us — the best of them do **not** rely on gradient
updates. This note records what those systems actually do (with sources), then maps each
mechanism onto a deterministic, weightless symbolic engine whose only durable state is an
append-only event log plus `data/seed/*.lino`.

All sources below were retrieved on **2026-07-20**.

## Reflexion — verbal reinforcement in an episodic memory buffer

Shinn, Cassano, Berman, Gopinath, Narasimhan and Yao, *Reflexion: Language Agents with
Verbal Reinforcement Learning* (arXiv:2303.11366, v1 20 Mar 2023, v4 10 Oct 2023).

Source: <https://arxiv.org/abs/2303.11366> (retrieved 2026-07-20)

What it actually does, per the abstract:

- It reinforces language agents **"not by updating weights, but instead through
  linguistic feedback"** — the framing is explicitly weightless, motivated by the fact
  that classic RL "require[s] extensive training samples and expensive model fine-tuning".
- Agents **"verbally reflect on task feedback signals, then maintain their own reflective
  text in an episodic memory buffer to induce better decision-making in subsequent
  trials"**. The reflection is a *durable artifact*, not a transient thought: it is what
  makes trial *n+1* differ from trial *n*.
- The feedback signal is deliberately polymorphic — "scalar values or free-form language",
  from "external or internally simulated" sources.
- Reported result: 91% pass@1 on HumanEval, versus a stated 80% for the GPT-4 baseline
  they compare against. (Quoted only as the paper's own claim.)

The transferable idea is not "write English notes". It is: **a failure must be converted
into a persisted, retrievable record that the next attempt is guaranteed to consult.**
Reflexion's episodic buffer is the mechanism that turns a failure into a changed future.

## Voyager — an ever-growing library of *verified* skills

Wang, Xie, Jiang, Mandlekar, Xiao, Zhu, Fan and Anandkumar, *Voyager: An Open-Ended
Embodied Agent with Large Language Models* (arXiv:2305.16291, submitted 25 May 2023,
revised 19 Oct 2023).

Sources: <https://arxiv.org/abs/2305.16291> and the project page
<https://voyager.minedojo.org/> (both retrieved 2026-07-20)

Voyager has exactly three components, and each maps to a stage of the adoption loop:

1. **An automatic curriculum that maximizes exploration** — see the next section.
2. **"An ever-growing skill library of executable code for storing and retrieving complex
   behaviors."** Skills are stored as executable programs; their natural-language
   descriptions act as the retrieval index, and a new task issues an embedding query that
   retrieves the top-5 relevant skills. Skills compose: simpler programs are called by
   more sophisticated ones, which the authors say "compounds the agent's abilities
   rapidly and alleviates catastrophic forgetting".
3. **An iterative prompting mechanism** that folds in environment feedback, execution
   errors, and **self-verification** — a critic pass that judges whether the generated
   program actually achieved the stated objective before it is kept.

Two properties matter for us. First, **nothing enters the library unverified** — the
self-verification step is the admission gate, and this is what makes the library an asset
rather than an accumulation of noise. Second, Voyager "interacts with GPT-4 via blackbox
queries, which bypasses the need for model parameter fine-tuning": again, capability
growth *without* touching weights. The authors also report that the learned skill library
transfers to a fresh Minecraft world to solve novel tasks from scratch — i.e. the adopted
artifact, not the model, carries the capability.

## Curriculum learning — ordering the frontier by difficulty

Bengio, Louradour, Collobert and Weston, *Curriculum Learning*, ICML 2009, pp. 41–48,
DOI 10.1145/1553374.1553380.

Sources: <https://icml.cc/Conferences/2009/abstracts.html> (abstract) and
<https://dblp.org/rec/conf/icml/BengioLCW09.html> (metadata) — both retrieved 2026-07-20.

From the ICML 2009 abstract: "Humans and animals learn much better when the examples are
not randomly presented but organized in a meaningful order which illustrates gradually
more concepts, and more complex ones. Here, we formalize such training strategies in the
context of machine learning, and call them 'curriculum learning'." The paper situates this
in the study of non-convex training criteria for neural networks and argues that selecting
and ordering training examples appropriately improves generalization.

Voyager's contribution is to make the curriculum *automatic and self-generated*: per the
project page, it considers exploration progress and the agent's state to propose tasks
sized to current capability, described as "an in-context form of novelty search" driven by
the instruction to discover as many diverse things as possible. This is the 2009 idea
without a human syllabus author — the agent reads its own frontier and orders it.

## Self-Refine (brief, for contrast)

Madaan et al., *Self-Refine: Iterative Refinement with Self-Feedback* (arXiv:2303.17651,
2023). Source: <https://arxiv.org/abs/2303.17651> (retrieved 2026-07-20). A single LLM
produces an output, critiques it, and refines it — with no additional training and no
external supervision. It is relevant as a boundary case: Self-Refine improves *one*
answer at test time but persists nothing across episodes. It is refinement without
adoption, and it is precisely the failure mode issue #701 is complaining about.

## Symbolic analogs adopted here

`formal-ai` is deterministic and weightless: answers are projections over an append-only
event log, and the only place a new capability can live is data/rules in
`data/seed/*.lino`. Each neural mechanism therefore has to be re-expressed as *durable
data plus a deterministic gate*.

| Neural mechanism | Symbolic analog adopted here |
| --- | --- |
| Reflexion's episodic reflection buffer | An **append-only durable frontier record**: every failure to answer (an `intent: unknown` event, an unanswered trending prompt, a failed benchmark case) and every failure to *adopt* is written as an event, not just logged. The next cycle reads the record deterministically instead of re-discovering the failure. |
| Voyager's verified skill library | **Validated candidate surfaces promoted into `data/seed/*.lino`** through the issue-#656 promotion protocol (`src/promotion.rs`): proposal → replayed gate batch → decision → `promotion_applied` / `promotion_rejection` events → seed edit written onto a `promotion/<run-id>` branch behind `--apply --confirm`. Human review of the pull request is the outer gate. |
| Voyager's self-verification critic | **Held-out tests and benchmark ratchets** as the admission gate. Verification is a replayed, digest-bound gate batch with manifest floors, and it fails closed when evidence is unparseable — an LLM critic's judgement is replaced by a reproducible pass/fail. |
| Voyager's automatic curriculum + Bengio et al.'s ordering | **Drive the frontier in difficulty order**: take unanswered prompt classes (Google Trends prompts, `intent: unknown` classes) ordered from cheapest-to-close to hardest, and keep going until the corpus is exhausted. The frontier is enumerable and finite, so the curriculum is a deterministic traversal rather than a novelty-search heuristic. |
| "Capability growth without fine-tuning" | Growth lands entirely in seed data and rules. The engine binary is unchanged by learning; a promoted `.lino` edit is the whole delta, and it is diffable, revertible, and reviewable. |
| Reflexion's cross-trial improvement claim | The **adoption contract**: a cycle is only successful if a previously-failing prompt now answers, proven by a test that failed before the seed edit and passes after. "Learned" without a behaviour delta is not learning. |

What explicitly does **not** transfer:

- **Gradient updates.** There are no weights. Nothing is "nudged"; a rule is either present
  in the seed corpus or it is not.
- **Sampling-based exploration.** Voyager's novelty search and Reflexion's re-rollouts rely
  on stochastic generation. Our frontier is enumerated from the event log, so exploration
  is a deterministic sweep, and identical inputs must produce identical promotion decisions.
- **Non-determinism / self-judged success.** An LLM critic saying "looks right" cannot be
  an admission gate here; only a replayed test batch can.
- **Embedding-based retrieval.** Voyager retrieves top-5 skills by embedding similarity.
  Seed rules are resolved by exact, explainable matching, since every answer must be a
  traceable projection.
- **Unbounded autonomy.** Voyager acquires skills "without human intervention"; here
  adoption terminates at a draft pull request awaiting human review.

## What we deliberately did not adopt

- **Free-form natural-language reflections as the carrier of learning.** Reflexion stores
  prose; prose cannot be a projection input for a deterministic engine. We store structured
  events and typed seed edits.
- **Self-verification by the same model that produced the candidate.** Self-Refine and
  Voyager both let the generator grade itself. We require an independent, replayable gate
  batch with held-out cases.
- **Direct writes to the corpus.** No loop writes `data/seed/` in place. Every adoption is
  a branch plus a reviewable diff — the protocol never pushes to the default branch.
- **Difficulty heuristics borrowed wholesale from curriculum learning.** Bengio et al.'s
  easy-to-hard ordering is adopted as an ordering principle; their loss-annealing
  formulation has no meaning without a training objective.
- **Open-ended novelty seeking.** Our objective is corpus exhaustion (close every known
  frontier item), not maximizing diversity of discoveries.
