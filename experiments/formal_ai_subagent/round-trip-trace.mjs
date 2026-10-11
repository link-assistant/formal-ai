#!/usr/bin/env node
// Prints every corpus sentence's round trip for one ordered pair (surviving /
// known terms, the forward and the backward text), one line each, so two
// trees can be diffed to find the sentences a seed change moved.
//
// Usage: node experiments/formal_ai_subagent/round-trip-trace.mjs <source> <target>
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
import { readCorpus } from '../../scripts/measure-round-trip-translation.mjs';

installTextHost();
const { roundTrip } = await import('../../js/agentic/crate/round_trip_translation.mjs');
const [source, target] = process.argv.slice(2);
for (const sentence of readCorpus().filter((entry) => entry.language === source)) {
  const trip = roundTrip(sentence.text, source, target);
  console.log(`${trip.survivingTerms}/${trip.knownTerms}\t${trip.forward}\t${trip.backward}`);
}
