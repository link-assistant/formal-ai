# Online research for issue #411

Research date: 2026-06-11.

## Sources checked

- Rasa rules documentation: <https://legacy-docs-oss.rasa.com/docs/rasa/rules/>
- Dialogflow ES training phrases documentation: <https://docs.cloud.google.com/dialogflow/es/docs/intents-training-phrases>
- Microsoft Bot Framework dialogs documentation: <https://learn.microsoft.com/en-us/azure/bot-service/bot-builder-concept-dialog?view=azure-bot-service-4.0>
- Botpress Library / intents documentation: <https://botpress.com/docs/studio/concepts/library>

## Notes

- Rasa treats rules as short conversation paths that should always follow the same behavior, but warns that rules alone do not generalize to unseen paths. This supports keeping the exact `Покажи правила` shortcut as a small deterministic rule while retaining the existing semantic role matcher for broader variants.
- Dialogflow models intent recognition with multiple training phrases per intent and recommends enough variation for real user expressions. The comparable formal-ai change is a balanced `short_rule_list` prompt-pattern group across every supported language, not a single Russian-only phrase.
- Bot Framework emphasizes persisted dialog state for multi-turn flows. The sort failure shown in the same issue body is a state/coreference defect and is covered by the existing issue-412 case study and tests; it is separate from this PR's behavior-rule-list shortcut.
- Botpress describes intents as user-message purpose groups with multiple utterances. The same idea maps cleanly to formal-ai's seed-backed intent `behavior_rules_list` plus checked prompt-pattern variations.

## Conclusion

The practical fix should be data-backed phrase coverage for the short behavior-rule-list request, with cross-language coverage gates so the Russian shortcut does not become an isolated special case. No upstream issue is needed: the defect is local to formal-ai's seed vocabulary and fallback mirrors.
