// Demo mode: the scripted greeting/feature turns and the playback hook that
// runs them in a dedicated demo conversation.

import React from "react";
import { EXAMPLE_PROMPTS } from "./application-constants.js";
import { messagesForConversation, randomInt } from "./conversations.jsx";

const { useEffect, useRef } = React;

// Issue #27 R5: the demo iterates through the same Example prompts list so
// every advertised feature is exercised. The greeting variants come from
// `EXAMPLE_PROMPTS` (`Greeting (...)` rows) and feature prompts are the
// remainder, minus actions that trigger downloads / file pickers.
const DEMO_GREETING_LABELS = new Set([
  "Greeting (en)",
  "Greeting (ru)",
  "Greeting (hi)",
  "Greeting (zh)",
]);

const DEMO_EXCLUDED_LABELS = new Set(["Export memory", "Import memory"]);

function demoGreetings() {
  return EXAMPLE_PROMPTS.filter((entry) => DEMO_GREETING_LABELS.has(entry.label));
}

function demoFeaturePrompts() {
  return EXAMPLE_PROMPTS.filter(
    (entry) =>
      !DEMO_GREETING_LABELS.has(entry.label) &&
      !DEMO_EXCLUDED_LABELS.has(entry.label),
  );
}

// Persistent cursors so each demo cycle advances through the lists rather
// than repeating the same prompts forever. Wraps when the cursor runs off
// the end.
let demoGreetingCursor = 0;

let demoFeatureCursor = 0;

function createDemoTurns() {
  const greetings = demoGreetings();
  const features = demoFeaturePrompts();
  const turns = [];
  if (greetings.length > 0) {
    const greeting = greetings[demoGreetingCursor % greetings.length];
    demoGreetingCursor = (demoGreetingCursor + 1) % greetings.length;
    turns.push({ text: greeting.text, label: greeting.label });
  }
  if (features.length > 0) {
    const feature = features[demoFeatureCursor % features.length];
    demoFeatureCursor = (demoFeatureCursor + 1) % features.length;
    turns.push({ text: feature.text, label: feature.label });
  }
  return turns;
}

function wait(milliseconds) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, milliseconds);
  });
}

// Plays the scripted demo conversation and restores the user's thread
// when demo mode is switched off.
export function useDemoPlayback({
  setMessages, setPending, demoMode, setDemoPhase, setDemoCountdown, currentConversationRef,
  conversationEventsRef, requestAnswer, appendUserMessage, appendAssistantMessage,
}) {
  // Issue #541 (R4): tracks whether the previous render was in demo mode so we
  // can detect the on→off transition and restore the user's real conversation
  // back into the UI. Demo writes go to a dedicated demo conversation (see
  // `demoConversationIdRef`), so restoration is a single IndexedDB lookup
  // against the still-pointed-at `currentConversationRef`.
  const demoWasActiveRef = useRef(demoMode);
  useEffect(() => {
    const wasActive = demoWasActiveRef.current;
    demoWasActiveRef.current = demoMode;

    if (!demoMode) {
      setDemoPhase("manual");
      setDemoCountdown(null);
      if (wasActive) {
        // Demo just turned off — restore whatever the user had open before
        // demo took over. If they had no prior conversation, fall back to an
        // empty composer rather than leaking the last demo turn into the UI.
        const userId = currentConversationRef.current;
        const cachedEvents = conversationEventsRef.current;
        const restored = userId
          ? messagesForConversation(cachedEvents, userId)
          : [];
        setMessages(restored);
        setPending(false);
      }
      return undefined;
    }

    let cancelled = false;
    let countdownTimer = 0;

    async function runCycle() {
      const turns = createDemoTurns();
      setMessages([]);
      setPending(true);
      setDemoPhase("playing");
      setDemoCountdown(null);

      for (const turn of turns) {
        if (cancelled) {
          return;
        }

        appendUserMessage(turn.text, { demoLabel: turn.label });
        await wait(randomInt(700, 1300));
        const answer = await requestAnswer(turn.text);
        if (cancelled) {
          return;
        }
        appendAssistantMessage(answer);
        await wait(randomInt(900, 1500));
      }

      setPending(false);
      const waitSeconds = randomInt(10, 20);
      let remainingSeconds = waitSeconds;
      setDemoPhase("waiting");
      setDemoCountdown(remainingSeconds);
      countdownTimer = window.setInterval(() => {
        remainingSeconds -= 1;
        if (remainingSeconds <= 0) {
          window.clearInterval(countdownTimer);
          if (!cancelled) {
            runCycle();
          }
          return;
        }
        setDemoCountdown(remainingSeconds);
      }, 1000);
    }

    runCycle();

    return () => {
      cancelled = true;
      window.clearInterval(countdownTimer);
      setPending(false);
    };
  }, [appendAssistantMessage, appendUserMessage, demoMode, requestAnswer]);
}
