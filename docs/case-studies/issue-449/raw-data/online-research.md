# Online research notes — arXiv:2605.00940 and its author

These notes collect the external facts gathered while analysing issue #449.
They are archived verbatim alongside the issue/PR snapshots so the case-study
README can cite a stable record. Quotations are paraphrased from the cited
sources; figures are taken from the paper's own text
(`paper-2605.00940-extracted.txt` in this folder).

## The paper

- **Title:** *Interpretable experiential learning based on state history and
  global feedback.*
- **Author:** Anton Kolonin.
- **Identifier:** `arXiv:2605.00940v1 [cs.LG]`, dated 1 May 2026.
- **Landing page:** <https://arxiv.org/abs/2605.00940>
- **Full-text HTML:** <https://arxiv.org/html/2605.00940v1>

### One-paragraph summary

The paper proposes a learning model that is interpretable by construction. A
behavioural model is represented as a **transition graph between sets of
states**; each transition is attributed with a **utility `U`** and an
**evidence count `C`**. Raw observations are first turned into interpretable
"objects, events and properties" by a *state transformer*, so no hidden or
latent states are introduced. Learning uses **global feedback**: at the end of
an episode the utility of every transition that was traversed is updated by the
episode-wide outcome, rather than gradually as in deep Q-learning. A decision
selects the next state by `argmax_s U` (when the *counted-utility* policy
`CU = False`) or by `argmax_s U·C` (when `CU = True`); a cosine-similarity
fallback is used when the current situation is not matched exactly but a stored
situation exceeds the state-similarity threshold `SS`.

### Three-layer architecture (paper §"Architecture")

1. **State transformer** — pre-processes raw input into interpretable
   objects/events/properties (for Breakout: ball and paddle positions, etc.).
2. **State learning layer** — an **in-memory graph database** storing the
   transition graph (states, transitions, utility, count).
3. **Decision-making layer** — selects the next state/action by the
   utility / counted-utility policy with the similarity fallback.

### Hyperparameters (paper §"Hyperparameters")

| Symbol | Name | Default | Meaning |
| --- | --- | --- | --- |
| CS | Context size | 2 | number of states forming a transition key |
| LM | Learning mode | 2 | 0 none / 1 positive / 2 positive+negative feedback |
| SR | State reward | True | whether positive/negative state reward is encoded |
| CU | Counted utility | False | decide by `argmax U` (False) or `argmax U·C` (True) |
| EA | Encode action | False | whether the action is part of the state encoding |
| SC | State count threshold | 2 | min experiences of a state series to be stored |
| SS | State similarity threshold | 0.9 | min cosine similarity for the fallback match |
| TU | Transition utility threshold | 0 | min accumulated utility for a transition to be a candidate |
| TC | Transition count threshold | 1 | min evidence count for a transition to be a candidate |

### Evaluation (paper §"Experiments"/"Results")

Evaluated on `BreakoutNoFrameskip-v4` (OpenAI Gym Atari Breakout). Reported
average scores used as baselines and results:

| Approach | Avg. Breakout score | Source cited by the paper |
| --- | --- | --- |
| Human | 31 | Mnih et al. (2013) |
| **This paper's model** | **120 @ 30M frames, 196 @ ~41M frames** | this paper |
| DQN | 168 | Mnih et al. (2013) |
| Rainbow-IQN | 176 | Toromanoff et al. (2019) |
| MuZero | (state-of-the-art, highest) | Schrittwieser et al. (2020) |

The headline claim is **competitive performance with deep-RL baselines while
running in real time on low-end hardware** and remaining fully interpretable
(every decision traces to stored transitions with their `U` and `C`).

## The author

- **Anton Kolonin** — founder of the **Aigents Group** (Novosibirsk, Russian
  Federation, since 2014), affiliated with the **SingularityNET Foundation**
  (Amsterdam, Netherlands) and **Novosibirsk State University** (AI Research
  Center).
- Works on "personal AI" and "agents of collective intelligence", unsupervised
  language learning, and a decentralised reputation-management system for
  SingularityNET.
- Advocates a **"horizontal neuro-symbolic integration"** approach to AGI and a
  representation-agnostic cognitive architecture.
- Profiles: <https://www.researchgate.net/profile/Anton-Kolonin>,
  <https://lifeboat.com/ex/bios.anton.kolonin>, <https://github.com/akolonin>.

## Related work and reference implementation

- **Prior paper (lineage of this approach):** Kolonin, *"Neuro-Symbolic
  Architecture for Experiential Learning in Discrete and Functional
  Environments"*, AGI 2021 / BICA, Springer LNCS.
  <https://link.springer.com/chapter/10.1007/978-3-030-93758-4_12>
- **Talk:** *"Experiential Learning from Sequential Data — applied to
  Reinforcement Learning"* (OpenCog AGI discussion).
  <https://www.youtube.com/watch?v=AV_QQ7fqalw>
- **Reference implementation / experiments:** the Aigents organisation on
  GitHub, in particular **`aigents/pygents`** ("Machine Learning experiments in
  Python for Aigents project"). <https://github.com/aigents/pygents>,
  <https://github.com/aigents>

## Why this is relevant to formal-ai (short version)

formal-ai is a deterministic, symbolic assistant that already stores
**Markov-style transition records** and **Bayesian evidence** as append-only
Links Notation in `src/probability.rs`, and already ranks candidates by a
softmax over `prior + accumulated utility`. The paper's distinctive additions —
keeping an **evidence count `C`** separate from utility `U`, deciding by
**`argmax U·C`** under a counted-utility switch, and gating **under-evidenced
transitions** with the `TU`/`TC` thresholds — are exactly the pieces the
existing module did not yet have. They port onto the associative stack as
additive, deterministic record/ranking logic with no neural inference, which is
precisely what the issue asks for ("apply all best practices from there, but use
our associative technological stack").

## Sources

- <https://arxiv.org/abs/2605.00940>
- <https://arxiv.org/html/2605.00940v1>
- <https://link.springer.com/chapter/10.1007/978-3-030-93758-4_12>
- <https://www.researchgate.net/profile/Anton-Kolonin>
- <https://lifeboat.com/ex/bios.anton.kolonin>
- <https://github.com/aigents/pygents>
- <https://github.com/aigents>
- <https://github.com/akolonin>
- <https://www.youtube.com/watch?v=AV_QQ7fqalw>
