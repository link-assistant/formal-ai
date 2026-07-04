# Issue 526 Online Research

## Round-Trip Translation

- ACL Anthology, *Rethinking Round-Trip Translation for Machine Translation
  Evaluation* (Findings ACL 2023):
  <https://aclanthology.org/2023.findings-acl.22/>

  The paper revisits round-trip translation for modern neural MT and reports
  that RTT can be useful for reference-free evaluation tasks such as predicting
  forward translation quality and identifying adversarial competitors. This
  supports using round-trip survival as a repository regression invariant.

- Aiken and Park, *The Efficacy of Round-trip Translation for MT Evaluation*
  (Translation Journal, 2010):
  <https://mt-archive.net/10/TranslationJ-2010-Aiken.pdf>

  Older empirical work is more cautious and warns against treating RTT as a
  complete standalone quality measure. This PR therefore uses RTT as a local
  formal-ai consistency test, not as a full human-quality metric.

## Reference-Based MT Metrics

- Papineni et al., *BLEU: a Method for Automatic Evaluation of Machine
  Translation* (ACL 2002): <https://aclanthology.org/P02-1040.pdf>

  BLEU established a fast reference-based metric for MT, but it depends on
  reference translations. Formal AI's issue #526 ask is different: it asks
  whether the system's own meta-language representation can survive a round
  trip.

- Rei et al., *COMET: A Neural Framework for MT Evaluation* (EMNLP 2020):
  <https://aclanthology.org/2020.emnlp-main.213.pdf>

  COMET is a learned evaluation framework that compares source, hypothesis, and
  reference. It is useful context, but it does not fit Formal AI's deterministic
  no-neural-inference regression suite.

## Interlingua / Meta-Language Translation

- Dorr, Hovy, and Levin, *Machine Translation: Interlingual Methods*:
  <https://www.umiacs.umd.edu/users/bonnie/Publications/Interlingual-MT-Dorr-Hovy-Levin.pdf>

  Interlingual MT translates through a language-neutral representation. That is
  the same architectural shape Formal AI documents as Links Notation / meaning
  projection.

- Dorr, *The use of lexical semantics in interlingual machine translation*:
  <https://link.springer.com/article/10.1007/BF00402510>

  The UNITRAN work demonstrates translating multiple natural languages through a
  lexical-semantic representation, reinforcing the design choice to make direct
  language-pair rendering subordinate to the shared meta representation.
