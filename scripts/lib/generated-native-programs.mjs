// Generated source-qualified native AST programs. Unknown algorithms remain refused.
export const GENERATED_NATIVE_PROGRAMS = [{"path":"formal_ai::summarization::apply_compound_words","kind":
"substitutionProgram","signature":["string","string"],"returnType":"string","program":{"selectorArgument":1,
"inputArgument":0,"alternatives":[{"selector":"ru","pairs":[["в которой ","где "],["для того чтобы ","чтобы "],[
"к примеру","например"]]},{"fallback":true,"pairs":[["in order to ","to "],["for the purpose of ","for "],[
"a number of ","several "],["user interface","UI"],["command line interface","CLI"],["artificial intelligence","AI"]]}]}
,"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::apply_semantic_primes","kind":"substitutionProgram","signature":["string","string"],
"returnType":"string","program":{"selectorArgument":1,"inputArgument":0,"alternatives":[{"selector":"ru","pairs":[[
"автоматизация","когда машина делает"],["оркестрирует","управляет вместе"],["делегирование","передача работы"],[
"детерминированный","всегда одинаковый"]]},{"fallback":true,"pairs":[["orchestrates","controls many"],[
"automation of automation","machine that makes other machines do"],["automation","machine doing"],["delegating",
"giving work to"],["deterministic","always the same"],["multilingual","in many languages"],["symbolic","rule-based"]]}]}
,"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::DEFAULT_MAX_STATEMENTS","kind":"nativeConstant","nativeType":"usize","value":30,"returnType":
"number","nativeSourceQualified":true,"sourceSpan":{"start":3100,"end":3145},"sourceWitnesses":[{"path":
"rust/src/summarization/mod.rs","sha256":"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::summarization::StatementKind","variant":"Misc","signature":[],"returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Identity","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Identity","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Purpose","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Purpose","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Language","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Language","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Stars","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Stars","returnType":"enum:formal_ai::summarization::StatementKind",
"nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},"sourceWitnesses":[{"path":
"rust/src/summarization/mod.rs","sha256":"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Feature","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Feature","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::UseCase","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"UseCase","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Install","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Install","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Example","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Example","returnType":
"enum:formal_ai::summarization::StatementKind","nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::Misc","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::StatementKind","variant":"Misc","returnType":"enum:formal_ai::summarization::StatementKind",
"nativeSourceQualified":true,"sourceSpan":{"start":3329,"end":4008},"sourceWitnesses":[{"path":
"rust/src/summarization/mod.rs","sha256":"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Standard","signature":[],"returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Identifier","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Identifier","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Topic","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Topic","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Short","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Short","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Standard","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Standard","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Full","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Full","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::Expand","kind":"nativeEnumVariant","enumPath":
"formal_ai::summarization::SummarizationMode","variant":"Expand","returnType":
"enum:formal_ai::summarization::SummarizationMode","nativeSourceQualified":true,"sourceSpan":{"start":7123,"end":8002},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::is_essential","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::summarization::StatementKind","signature":["enum:formal_ai::summarization::StatementKind"],"name":
"is_essential","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Identity","value":true},{"variant":
"Purpose","value":true},{"variant":"Language","value":true},{"variant":"Stars","value":true},{"variant":"Feature",
"value":false},{"variant":"UseCase","value":false},{"variant":"Install","value":false},{"variant":"Example","value":
false},{"variant":"Misc","value":false}],"nativeSourceQualified":true,"sourceSpan":{"start":4010,"end":6227},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::StatementKind::is_boilerplate","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::summarization::StatementKind","signature":["enum:formal_ai::summarization::StatementKind"],"name":
"is_boilerplate","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Identity","value":false},{"variant":
"Purpose","value":false},{"variant":"Language","value":false},{"variant":"Stars","value":false},{"variant":"Feature",
"value":false},{"variant":"UseCase","value":false},{"variant":"Install","value":true},{"variant":"Example","value":true}
,{"variant":"Misc","value":false}],"nativeSourceQualified":true,"sourceSpan":{"start":4010,"end":6227},"sourceWitnesses"
:[{"path":"rust/src/summarization/mod.rs","sha256":"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{
"path":"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::target_percent","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::summarization::SummarizationMode","signature":["enum:formal_ai::summarization::SummarizationMode"],"name":
"target_percent","nativeType":"u32","returnType":"number","cases":[{"variant":"Identifier","value":0},{"variant":"Topic"
,"value":0},{"variant":"Short","value":20},{"variant":"Standard","value":50},{"variant":"Full","value":100},{"variant":
"Expand","value":200}],"nativeSourceQualified":true,"sourceSpan":{"start":8004,"end":9742},"sourceWitnesses":[{"path":
"rust/src/summarization/mod.rs","sha256":"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::summarization::SummarizationMode::is_label_only","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::summarization::SummarizationMode","signature":["enum:formal_ai::summarization::SummarizationMode"],"name":
"is_label_only","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Identifier","value":true},{"variant":
"Topic","value":true},{"variant":"Short","value":false},{"variant":"Standard","value":false},{"variant":"Full","value":
false},{"variant":"Expand","value":false}],"nativeSourceQualified":true,"sourceSpan":{"start":8004,"end":9742},
"sourceWitnesses":[{"path":"rust/src/summarization/mod.rs","sha256":
"9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::meta_construction::RecursionMode","variant":"Both","signature":[],"returnType":
"enum:formal_ai::meta_construction::RecursionMode","nativeSourceQualified":true,"sourceSpan":{"start":2330,"end":2646},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::Down","kind":"nativeEnumVariant","enumPath":
"formal_ai::meta_construction::RecursionMode","variant":"Down","returnType":
"enum:formal_ai::meta_construction::RecursionMode","nativeSourceQualified":true,"sourceSpan":{"start":2330,"end":2646},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::Up","kind":"nativeEnumVariant","enumPath":
"formal_ai::meta_construction::RecursionMode","variant":"Up","returnType":
"enum:formal_ai::meta_construction::RecursionMode","nativeSourceQualified":true,"sourceSpan":{"start":2330,"end":2646},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::Both","kind":"nativeEnumVariant","enumPath":
"formal_ai::meta_construction::RecursionMode","variant":"Both","returnType":
"enum:formal_ai::meta_construction::RecursionMode","nativeSourceQualified":true,"sourceSpan":{"start":2330,"end":2646},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::emits_downward","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::meta_construction::RecursionMode","signature":["enum:formal_ai::meta_construction::RecursionMode"],"name":
"emits_downward","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Down","value":true},{"variant":"Up",
"value":false},{"variant":"Both","value":true}],"nativeSourceQualified":true,"sourceSpan":{"start":2648,"end":3658},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::emits_upward","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::meta_construction::RecursionMode","signature":["enum:formal_ai::meta_construction::RecursionMode"],"name":
"emits_upward","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Down","value":false},{"variant":"Up",
"value":true},{"variant":"Both","value":true}],"nativeSourceQualified":true,"sourceSpan":{"start":2648,"end":3658},
"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::meta_self_improvement::SelfImprovementMode","variant":"Propose","signature":[],"returnType":
"enum:formal_ai::meta_self_improvement::SelfImprovementMode","nativeSourceQualified":true,"sourceSpan":{"start":2650,
"end":2925},"sourceWitnesses":[{"path":"rust/src/meta_self_improvement.rs","sha256":
"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::Off","kind":"nativeEnumVariant","enumPath":
"formal_ai::meta_self_improvement::SelfImprovementMode","variant":"Off","returnType":
"enum:formal_ai::meta_self_improvement::SelfImprovementMode","nativeSourceQualified":true,"sourceSpan":{"start":2650,
"end":2925},"sourceWitnesses":[{"path":"rust/src/meta_self_improvement.rs","sha256":
"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::Propose","kind":"nativeEnumVariant","enumPath":
"formal_ai::meta_self_improvement::SelfImprovementMode","variant":"Propose","returnType":
"enum:formal_ai::meta_self_improvement::SelfImprovementMode","nativeSourceQualified":true,"sourceSpan":{"start":2650,
"end":2925},"sourceWitnesses":[{"path":"rust/src/meta_self_improvement.rs","sha256":
"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::proposes","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::meta_self_improvement::SelfImprovementMode","signature":[
"enum:formal_ai::meta_self_improvement::SelfImprovementMode"],"name":"proposes","nativeType":"bool","returnType":
"boolean","cases":[{"variant":"Off","value":false},{"variant":"Propose","value":true}],"nativeSourceQualified":true,
"sourceSpan":{"start":2927,"end":3670},"sourceWitnesses":[{"path":"rust/src/meta_self_improvement.rs","sha256":
"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::selection::SelectionMode::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::selection::SelectionMode","variant":"Record","signature":[],"returnType":
"enum:formal_ai::selection::SelectionMode","nativeSourceQualified":true,"sourceSpan":{"start":1899,"end":2116},
"sourceWitnesses":[{"path":"rust/src/selection.rs","sha256":
"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":"formal_ai::selection::SelectionMode::Off"
,"kind":"nativeEnumVariant","enumPath":"formal_ai::selection::SelectionMode","variant":"Off","returnType":
"enum:formal_ai::selection::SelectionMode","nativeSourceQualified":true,"sourceSpan":{"start":1899,"end":2116},
"sourceWitnesses":[{"path":"rust/src/selection.rs","sha256":
"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::selection::SelectionMode::Record","kind":"nativeEnumVariant","enumPath":
"formal_ai::selection::SelectionMode","variant":"Record","returnType":"enum:formal_ai::selection::SelectionMode",
"nativeSourceQualified":true,"sourceSpan":{"start":1899,"end":2116},"sourceWitnesses":[{"path":"rust/src/selection.rs",
"sha256":"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::selection::SelectionMode::emits_artifact","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::selection::SelectionMode","signature":["enum:formal_ai::selection::SelectionMode"],"name":"emits_artifact",
"nativeType":"bool","returnType":"boolean","cases":[{"variant":"Off","value":false},{"variant":"Record","value":true}],
"nativeSourceQualified":true,"sourceSpan":{"start":2118,"end":2860},"sourceWitnesses":[{"path":"rust/src/selection.rs",
"sha256":"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::skill_ledger::SkillMode::default","kind":"nativeEnumDefault","enumPath":"formal_ai::skill_ledger::SkillMode"
,"variant":"Accumulate","signature":[],"returnType":"enum:formal_ai::skill_ledger::SkillMode","nativeSourceQualified":
true,"sourceSpan":{"start":2439,"end":2678},"sourceWitnesses":[{"path":"rust/src/skill_ledger.rs","sha256":
"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":"formal_ai::skill_ledger::SkillMode::Off",
"kind":"nativeEnumVariant","enumPath":"formal_ai::skill_ledger::SkillMode","variant":"Off","returnType":
"enum:formal_ai::skill_ledger::SkillMode","nativeSourceQualified":true,"sourceSpan":{"start":2439,"end":2678},
"sourceWitnesses":[{"path":"rust/src/skill_ledger.rs","sha256":
"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::skill_ledger::SkillMode::Accumulate","kind":"nativeEnumVariant","enumPath":
"formal_ai::skill_ledger::SkillMode","variant":"Accumulate","returnType":"enum:formal_ai::skill_ledger::SkillMode",
"nativeSourceQualified":true,"sourceSpan":{"start":2439,"end":2678},"sourceWitnesses":[{"path":
"rust/src/skill_ledger.rs","sha256":"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::skill_ledger::SkillMode::emits_ledger","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::skill_ledger::SkillMode","signature":["enum:formal_ai::skill_ledger::SkillMode"],"name":"emits_ledger",
"nativeType":"bool","returnType":"boolean","cases":[{"variant":"Off","value":false},{"variant":"Accumulate","value":true
}],"nativeSourceQualified":true,"sourceSpan":{"start":2680,"end":3426},"sourceWitnesses":[{"path":
"rust/src/skill_ledger.rs","sha256":"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::world_model_dialog::WorldModelMode::default","kind":"nativeEnumDefault","enumPath":
"formal_ai::world_model_dialog::WorldModelMode","variant":"Off","signature":[],"returnType":
"enum:formal_ai::world_model_dialog::WorldModelMode","nativeSourceQualified":true,"sourceSpan":{"start":3768,"end":4017}
,"sourceWitnesses":[{"path":"rust/src/world_model_dialog.rs","sha256":
"4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::world_model_dialog::WorldModelMode::Off","kind":"nativeEnumVariant","enumPath":
"formal_ai::world_model_dialog::WorldModelMode","variant":"Off","returnType":
"enum:formal_ai::world_model_dialog::WorldModelMode","nativeSourceQualified":true,"sourceSpan":{"start":3768,"end":4017}
,"sourceWitnesses":[{"path":"rust/src/world_model_dialog.rs","sha256":
"4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::world_model_dialog::WorldModelMode::Track","kind":"nativeEnumVariant","enumPath":
"formal_ai::world_model_dialog::WorldModelMode","variant":"Track","returnType":
"enum:formal_ai::world_model_dialog::WorldModelMode","nativeSourceQualified":true,"sourceSpan":{"start":3768,"end":4017}
,"sourceWitnesses":[{"path":"rust/src/world_model_dialog.rs","sha256":
"4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::world_model_dialog::WorldModelMode::emits_artifact","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::world_model_dialog::WorldModelMode","signature":["enum:formal_ai::world_model_dialog::WorldModelMode"],
"name":"emits_artifact","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Off","value":false},{"variant":
"Track","value":true}],"nativeSourceQualified":true,"sourceSpan":{"start":4019,"end":4776},"sourceWitnesses":[{"path":
"rust/src/world_model_dialog.rs","sha256":"4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"recordPath":
"formal_ai::selection_heuristics::ActionCost","fields":{"steps":"number","code_size":"number","resource_units":"number",
"leaf_count":"number"},"nativeFields":{"steps":{"kind":"integer","native":"u32","type":"number","copy":true},"code_size"
:{"kind":"integer","native":"usize","type":"number","copy":true},"resource_units":{"kind":"integer","native":"u64",
"type":"number","copy":true},"leaf_count":{"kind":"integer","native":"u32","type":"number","copy":true}},"copy":true,
"sourceWitnesses":[{"path":"rust/src/selection_heuristics.rs","sha256":
"3aec43ef1dcaa1cd31532edc4741bf1d43c3456eeade30f3a97d80c1f00d36a0"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":true,
"nativeRecordQualified":true,"path":"formal_ai::selection_heuristics::ActionCost","kind":"nativeRecordConstructor",
"returnType":"record:formal_ai::selection_heuristics::ActionCost","sourceSpan":{"start":5998,"end":6465}},{"recordPath":
"formal_ai::selection_heuristics::ActionCost","fields":{"steps":"number","code_size":"number","resource_units":"number",
"leaf_count":"number"},"nativeFields":{"steps":{"kind":"integer","native":"u32","type":"number","copy":true},"code_size"
:{"kind":"integer","native":"usize","type":"number","copy":true},"resource_units":{"kind":"integer","native":"u64",
"type":"number","copy":true},"leaf_count":{"kind":"integer","native":"u32","type":"number","copy":true}},"copy":true,
"sourceWitnesses":[{"path":"rust/src/selection_heuristics.rs","sha256":
"3aec43ef1dcaa1cd31532edc4741bf1d43c3456eeade30f3a97d80c1f00d36a0"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":true,
"nativeRecordQualified":true,"path":"formal_ai::selection_heuristics::ActionCost::default","kind":"nativeRecordDefault",
"signature":[],"returnType":"record:formal_ai::selection_heuristics::ActionCost","defaults":{"steps":0,"code_size":0,
"resource_units":0,"leaf_count":0},"sourceSpan":{"start":5998,"end":6465}},{"recordPath":
"formal_ai::selection_heuristics::CandidateScore","fields":{"candidate_id":"string","checks":
"tuple:[\"number\",\"number\"]","cost":"record:formal_ai::selection_heuristics::ActionCost"},"nativeFields":{
"candidate_id":{"kind":"string","type":"string","copy":false},"checks":{"kind":"tuple","elements":[{"kind":"integer",
"native":"usize","type":"number","copy":true},{"kind":"integer","native":"usize","type":"number","copy":true}],"type":
"tuple:[\"number\",\"number\"]","copy":true},"cost":{"kind":"record","name":"ActionCost","type":
"record:formal_ai::selection_heuristics::ActionCost","copy":true}},"copy":false,"sourceWitnesses":[{"path":
"rust/src/selection_heuristics.rs","sha256":"3aec43ef1dcaa1cd31532edc4741bf1d43c3456eeade30f3a97d80c1f00d36a0"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":
true,"nativeRecordQualified":true,"path":"formal_ai::selection_heuristics::CandidateScore","kind":
"nativeRecordConstructor","returnType":"record:formal_ai::selection_heuristics::CandidateScore","sourceSpan":{"start":
6548,"end":6911}},{"recordPath":"formal_ai::selection_heuristics::CandidateScore","fields":{"candidate_id":"string",
"checks":"tuple:[\"number\",\"number\"]","cost":"record:formal_ai::selection_heuristics::ActionCost"},"nativeFields":{
"candidate_id":{"kind":"string","type":"string","copy":false},"checks":{"kind":"tuple","elements":[{"kind":"integer",
"native":"usize","type":"number","copy":true},{"kind":"integer","native":"usize","type":"number","copy":true}],"type":
"tuple:[\"number\",\"number\"]","copy":true},"cost":{"kind":"record","name":"ActionCost","type":
"record:formal_ai::selection_heuristics::ActionCost","copy":true}},"copy":false,"sourceWitnesses":[{"path":
"rust/src/selection_heuristics.rs","sha256":"3aec43ef1dcaa1cd31532edc4741bf1d43c3456eeade30f3a97d80c1f00d36a0"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":
true,"nativeRecordQualified":true,"path":"formal_ai::selection_heuristics::CandidateScore::satisfies","kind":
"nativeRecordBorrow","signature":["record:formal_ai::selection_heuristics::CandidateScore"],"returnType":"boolean",
"body":{"kind":"binary","operator":"&&","left":{"kind":"binary","operator":">","left":{"kind":"projection","path":[
"checks","1"],"type":"number","native":"usize"},"right":{"kind":"literal","value":0,"type":"number"},"type":"boolean"},
"right":{"kind":"binary","operator":"==","left":{"kind":"projection","path":["checks","0"],"type":"number","native":
"usize"},"right":{"kind":"projection","path":["checks","1"],"type":"number","native":"usize"},"type":"boolean"},"type":
"boolean"},"sourceSpan":{"start":7101,"end":7222}},{"recordPath":"formal_ai::skill_ledger::PromotionGate","fields":{
"has_tests":"boolean","has_benchmark_delta":"boolean"},"nativeFields":{"has_tests":{"kind":"boolean","type":"boolean",
"copy":true},"has_benchmark_delta":{"kind":"boolean","type":"boolean","copy":true}},"copy":true,"sourceWitnesses":[{
"path":"rust/src/skill_ledger.rs","sha256":"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":
true,"nativeRecordQualified":true,"path":"formal_ai::skill_ledger::PromotionGate","kind":"nativeRecordConstructor",
"returnType":"record:formal_ai::skill_ledger::PromotionGate","sourceSpan":{"start":5125,"end":5382}},{"recordPath":
"formal_ai::skill_ledger::PromotionGate","fields":{"has_tests":"boolean","has_benchmark_delta":"boolean"},"nativeFields"
:{"has_tests":{"kind":"boolean","type":"boolean","copy":true},"has_benchmark_delta":{"kind":"boolean","type":"boolean",
"copy":true}},"copy":true,"sourceWitnesses":[{"path":"rust/src/skill_ledger.rs","sha256":
"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":true,
"nativeRecordQualified":true,"path":"formal_ai::skill_ledger::PromotionGate::default","kind":"nativeRecordDefault",
"signature":[],"returnType":"record:formal_ai::skill_ledger::PromotionGate","defaults":{"has_tests":false,
"has_benchmark_delta":false},"sourceSpan":{"start":5125,"end":5382}},{"recordPath":
"formal_ai::skill_ledger::PromotionGate","fields":{"has_tests":"boolean","has_benchmark_delta":"boolean"},"nativeFields"
:{"has_tests":{"kind":"boolean","type":"boolean","copy":true},"has_benchmark_delta":{"kind":"boolean","type":"boolean",
"copy":true}},"copy":true,"sourceWitnesses":[{"path":"rust/src/skill_ledger.rs","sha256":
"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"nativeSourceQualified":true,
"nativeRecordQualified":true,"path":"formal_ai::skill_ledger::PromotionGate::satisfied","kind":"nativeRecordCopyRead",
"signature":["record:formal_ai::skill_ledger::PromotionGate"],"returnType":"boolean","body":{"kind":"binary","operator":
"&&","left":{"kind":"projection","path":["has_tests"],"type":"boolean"},"right":{"kind":"projection","path":[
"has_benchmark_delta"],"type":"boolean"},"type":"boolean"},"sourceSpan":{"start":5495,"end":5606}},{"path":
"formal_ai::cue_lexicon::CueMatch::Token","kind":"nativeEnumVariant","enumPath":"formal_ai::cue_lexicon::CueMatch",
"variant":"Token","returnType":"enum:formal_ai::cue_lexicon::CueMatch","nativeSourceQualified":true,"sourceSpan":{
"start":1334,"end":1656},"sourceWitnesses":[{"path":"rust/src/cue_lexicon.rs","sha256":
"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::cue_lexicon::CueMatch::Substring","kind":"nativeEnumVariant","enumPath":"formal_ai::cue_lexicon::CueMatch",
"variant":"Substring","returnType":"enum:formal_ai::cue_lexicon::CueMatch","nativeSourceQualified":true,"sourceSpan":{
"start":1334,"end":1656},"sourceWitnesses":[{"path":"rust/src/cue_lexicon.rs","sha256":
"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":"formal_ai::cue_lexicon::CueMatch::Prefix"
,"kind":"nativeEnumVariant","enumPath":"formal_ai::cue_lexicon::CueMatch","variant":"Prefix","returnType":
"enum:formal_ai::cue_lexicon::CueMatch","nativeSourceQualified":true,"sourceSpan":{"start":1334,"end":1656},
"sourceWitnesses":[{"path":"rust/src/cue_lexicon.rs","sha256":
"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":"formal_ai::cue_lexicon::CueMatch::slug",
"name":"slug","kind":"nativeEnumStringMatch","signature":["enum:formal_ai::cue_lexicon::CueMatch"],"returnType":"string"
,"cases":[{"variant":"Token","value":"token"},{"variant":"Substring","value":"substring"},{"variant":"Prefix","value":
"prefix"}],"enumPath":"formal_ai::cue_lexicon::CueMatch","nativeSourceQualified":true,"nativeOptionQualified":true,
"sourceSpan":{"start":1658,"end":2599},"sourceWitnesses":[{"path":"rust/src/cue_lexicon.rs","sha256":
"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::cue_lexicon::CueMatch::from_slug","name":"from_slug","kind":"nativeOptionStringParse","signature":["string"]
,"returnType":"option:enum:formal_ai::cue_lexicon::CueMatch","normalized":false,"cases":[{"input":"token","variant":
"Token"},{"input":"substring","variant":"Substring"},{"input":"prefix","variant":"Prefix"}],"enumPath":
"formal_ai::cue_lexicon::CueMatch","nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":1658,
"end":2599},"sourceWitnesses":[{"path":"rust/src/cue_lexicon.rs","sha256":
"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::slug","name":"slug","kind":"nativeEnumStringMatch","signature":[
"enum:formal_ai::meta_construction::RecursionMode"],"returnType":"string","cases":[{"variant":"Down","value":"down"},{
"variant":"Up","value":"up"},{"variant":"Both","value":"both"}],"enumPath":"formal_ai::meta_construction::RecursionMode"
,"nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":2648,"end":3658},"sourceWitnesses":[{
"path":"rust/src/meta_construction.rs","sha256":"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{
"path":"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_construction::RecursionMode::from_slug","name":"from_slug","kind":"nativeOptionStringParse","signature"
:["string"],"returnType":"option:enum:formal_ai::meta_construction::RecursionMode","normalized":true,"cases":[{"input":
"down","variant":"Down"},{"input":"up","variant":"Up"},{"input":"both","variant":"Both"}],"enumPath":
"formal_ai::meta_construction::RecursionMode","nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{
"start":2648,"end":3658},"sourceWitnesses":[{"path":"rust/src/meta_construction.rs","sha256":
"be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::slug","name":"slug","kind":"nativeEnumStringMatch","signature":[
"enum:formal_ai::meta_self_improvement::SelfImprovementMode"],"returnType":"string","cases":[{"variant":"Off","value":
"off"},{"variant":"Propose","value":"propose"}],"enumPath":"formal_ai::meta_self_improvement::SelfImprovementMode",
"nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":2927,"end":3670},"sourceWitnesses":[{
"path":"rust/src/meta_self_improvement.rs","sha256":"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},
{"path":"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::meta_self_improvement::SelfImprovementMode::from_slug","name":"from_slug","kind":"nativeOptionStringParse",
"signature":["string"],"returnType":"option:enum:formal_ai::meta_self_improvement::SelfImprovementMode","normalized":
true,"cases":[{"input":"off","variant":"Off"},{"input":"propose","variant":"Propose"}],"enumPath":
"formal_ai::meta_self_improvement::SelfImprovementMode","nativeSourceQualified":true,"nativeOptionQualified":true,
"sourceSpan":{"start":2927,"end":3670},"sourceWitnesses":[{"path":"rust/src/meta_self_improvement.rs","sha256":
"efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::selection::SelectionMode::slug","name":"slug","kind":"nativeEnumStringMatch","signature":[
"enum:formal_ai::selection::SelectionMode"],"returnType":"string","cases":[{"variant":"Off","value":"off"},{"variant":
"Record","value":"record"}],"enumPath":"formal_ai::selection::SelectionMode","nativeSourceQualified":true,
"nativeOptionQualified":true,"sourceSpan":{"start":2118,"end":2860},"sourceWitnesses":[{"path":"rust/src/selection.rs",
"sha256":"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::selection::SelectionMode::from_slug","name":"from_slug","kind":"nativeOptionStringParse","signature":[
"string"],"returnType":"option:enum:formal_ai::selection::SelectionMode","normalized":true,"cases":[{"input":"off",
"variant":"Off"},{"input":"record","variant":"Record"}],"enumPath":"formal_ai::selection::SelectionMode",
"nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":2118,"end":2860},"sourceWitnesses":[{
"path":"rust/src/selection.rs","sha256":"10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::service_accessibility::ServiceStatus::Reachable","kind":"nativeEnumVariant","enumPath":
"formal_ai::service_accessibility::ServiceStatus","variant":"Reachable","returnType":
"enum:formal_ai::service_accessibility::ServiceStatus","nativeSourceQualified":true,"sourceSpan":{"start":3207,"end":
3437},"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::service_accessibility::ServiceStatus::Unreachable","kind":"nativeEnumVariant","enumPath":
"formal_ai::service_accessibility::ServiceStatus","variant":"Unreachable","returnType":
"enum:formal_ai::service_accessibility::ServiceStatus","nativeSourceQualified":true,"sourceSpan":{"start":3207,"end":
3437},"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::service_accessibility::ServiceStatus::is_reachable","kind":"nativeEnumScalarMatch","enumPath":
"formal_ai::service_accessibility::ServiceStatus","signature":["enum:formal_ai::service_accessibility::ServiceStatus"],
"name":"is_reachable","nativeType":"bool","returnType":"boolean","cases":[{"variant":"Reachable","value":true},{
"variant":"Unreachable","value":false}],"nativeSourceQualified":true,"sourceSpan":{"start":3439,"end":4240},
"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::service_accessibility::ServiceStatus::slug","name":"slug","kind":"nativeEnumStringMatch","signature":[
"enum:formal_ai::service_accessibility::ServiceStatus"],"returnType":"string","cases":[{"variant":"Reachable","value":
"reachable"},{"variant":"Unreachable","value":"unreachable"}],"enumPath":
"formal_ai::service_accessibility::ServiceStatus","nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan"
:{"start":3439,"end":4240},"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::service_accessibility::ServiceStatus::from_slug","name":"from_slug","kind":"nativeOptionStringParse",
"signature":["string"],"returnType":"option:enum:formal_ai::service_accessibility::ServiceStatus","normalized":false,
"cases":[{"input":"reachable","variant":"Reachable"},{"input":"unreachable","variant":"Unreachable"}],"enumPath":
"formal_ai::service_accessibility::ServiceStatus","nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan"
:{"start":3439,"end":4240},"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":"formal_ai::skill_ledger::SkillMode::slug"
,"name":"slug","kind":"nativeEnumStringMatch","signature":["enum:formal_ai::skill_ledger::SkillMode"],"returnType":
"string","cases":[{"variant":"Off","value":"off"},{"variant":"Accumulate","value":"accumulate"}],"enumPath":
"formal_ai::skill_ledger::SkillMode","nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":
2680,"end":3426},"sourceWitnesses":[{"path":"rust/src/skill_ledger.rs","sha256":
"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::skill_ledger::SkillMode::from_slug","name":"from_slug","kind":"nativeOptionStringParse","signature":[
"string"],"returnType":"option:enum:formal_ai::skill_ledger::SkillMode","normalized":true,"cases":[{"input":"off",
"variant":"Off"},{"input":"accumulate","variant":"Accumulate"}],"enumPath":"formal_ai::skill_ledger::SkillMode",
"nativeSourceQualified":true,"nativeOptionQualified":true,"sourceSpan":{"start":2680,"end":3426},"sourceWitnesses":[{
"path":"rust/src/skill_ledger.rs","sha256":"80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}]},{"path":
"formal_ai::anticipation::ANTICIPATION_FRONTIER","canonicalPath":"formal_ai::anticipation::ANTICIPATION_FRONTIER","kind"
:"nativeConstant","nativeType":"&str","returnType":"string","value":"anticipation","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1179,"end":1234},"sourceWitnesses":[{"path":
"rust/src/anticipation.rs","sha256":"0b62bfa63c903e51dc53febad2dacb1c2409bfbf08553860ed2eac112241e951"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant ANTICIPATION_FRONTIER (exported) (type string) (value (string anticipation)))","maintainedJavaScript":
"export const ANTICIPATION_FRONTIER = 'anticipation';","maintainedTranslationSha256":
"c10c0e5ec8f025aa5d859de5692f66d6d132b11e7c85918634465030e5410e4f"},{"path":
"formal_ai::anticipation::ANTICIPATION_PREDICTION_KIND","canonicalPath":
"formal_ai::anticipation::ANTICIPATION_PREDICTION_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"anticipation_prediction","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":1235,"end":1308},"sourceWitnesses":[{"path":"rust/src/anticipation.rs","sha256":
"0b62bfa63c903e51dc53febad2dacb1c2409bfbf08553860ed2eac112241e951"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant ANTICIPATION_PREDICTION_KIND (exported) (type string) (value (string anticipation_prediction)))",
"maintainedJavaScript":"export const ANTICIPATION_PREDICTION_KIND = 'anticipation_prediction';",
"maintainedTranslationSha256":"cccecea7abec15d4b9c169cd29fc9186334c193e40a9ebb5146e0ced9f04b609"},{"path":
"formal_ai::anticipation::ANTICIPATION_SOURCE_KIND","canonicalPath":"formal_ai::anticipation::ANTICIPATION_SOURCE_KIND",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"anticipation_source","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1309,"end":1374},"sourceWitnesses":[{"path":
"rust/src/anticipation.rs","sha256":"0b62bfa63c903e51dc53febad2dacb1c2409bfbf08553860ed2eac112241e951"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant ANTICIPATION_SOURCE_KIND (exported) (type string) (value (string anticipation_source)))",
"maintainedJavaScript":"export const ANTICIPATION_SOURCE_KIND = 'anticipation_source';","maintainedTranslationSha256":
"11e3c6e64b13f577c7e8f2ec15cc901dba6bae350a5d216da563778669e35225"},{"path":
"formal_ai::anticipation::PREDICTION_HIT_KIND","canonicalPath":"formal_ai::anticipation::PREDICTION_HIT_KIND","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"prediction_hit","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1375,"end":1430},"sourceWitnesses":[{"path":
"rust/src/anticipation.rs","sha256":"0b62bfa63c903e51dc53febad2dacb1c2409bfbf08553860ed2eac112241e951"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant PREDICTION_HIT_KIND (exported) (type string) (value (string prediction_hit)))","maintainedJavaScript":
"export const PREDICTION_HIT_KIND = 'prediction_hit';","maintainedTranslationSha256":
"6ba60807837d1a3729473d1964b64e0999422e6a463032b46508f4393dc2e313"},{"path":
"formal_ai::bounded_autonomy::AUTONOMY_MODE_VARIABLE","canonicalPath":
"formal_ai::bounded_autonomy::AUTONOMY_MODE_VARIABLE","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"FORMAL_AI_AUTONOMY","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":
2136,"end":2198},"sourceWitnesses":[{"path":"rust/src/bounded_autonomy.rs","sha256":
"df8354b549baefbe59a705f480766a09c945e3a3cc2099da9b936d03e6f272d8"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant AUTONOMY_MODE_VARIABLE (exported) (type string) (value (string FORMAL_AI_AUTONOMY)))","maintainedJavaScript":
"export const AUTONOMY_MODE_VARIABLE = 'FORMAL_AI_AUTONOMY';","maintainedTranslationSha256":
"5f67ed0a5ab288b73743d8a42e90d205cf8f7d63606eeb8d56c9c422ebf63286"},{"path":
"formal_ai::bounded_autonomy::FULL_AUTONOMOUS_VALUE","canonicalPath":
"formal_ai::bounded_autonomy::FULL_AUTONOMOUS_VALUE","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"full","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2276,"end":2323
},"sourceWitnesses":[{"path":"rust/src/bounded_autonomy.rs","sha256":
"df8354b549baefbe59a705f480766a09c945e3a3cc2099da9b936d03e6f272d8"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FULL_AUTONOMOUS_VALUE (exported) (type string) (value (string full)))","maintainedJavaScript":
"export const FULL_AUTONOMOUS_VALUE = 'full';","maintainedTranslationSha256":
"971c65c376241ab5c28419dbc3c2fbfefa92ff0050851850532226f05e271b63"},{"path":
"formal_ai::bounded_autonomy::FULL_TRUST_VARIABLE","canonicalPath":"formal_ai::bounded_autonomy::FULL_TRUST_VARIABLE",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"FORMAL_AI_FULL_TRUST","nativeSourceQualified"
:true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2382,"end":2443},"sourceWitnesses":[{"path":
"rust/src/bounded_autonomy.rs","sha256":"df8354b549baefbe59a705f480766a09c945e3a3cc2099da9b936d03e6f272d8"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FULL_TRUST_VARIABLE (exported) (type string) (value (string FORMAL_AI_FULL_TRUST)))","maintainedJavaScript":
"export const FULL_TRUST_VARIABLE = 'FORMAL_AI_FULL_TRUST';","maintainedTranslationSha256":
"1c6c19c7c6146d107f4cb6419cbdcc4fec9f6ead1f020c3a04ad3bfe8de37554"},{"path":
"formal_ai::bounded_autonomy::FULL_TRUST_VALUE","canonicalPath":"formal_ai::bounded_autonomy::FULL_TRUST_VALUE","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"1","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2514,"end":2553},"sourceWitnesses":[{"path":
"rust/src/bounded_autonomy.rs","sha256":"df8354b549baefbe59a705f480766a09c945e3a3cc2099da9b936d03e6f272d8"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FULL_TRUST_VALUE (exported) (type string) (value (string 1)))","maintainedJavaScript":
"export const FULL_TRUST_VALUE = '1';","maintainedTranslationSha256":
"6451a4caeaab18d5219f51c6593b1701e5e69c58014924e980c06bcf4dba304b"},{"path":
"formal_ai::bounded_autonomy::STUCK_RECOVERY_LIMIT_VARIABLE","canonicalPath":
"formal_ai::bounded_autonomy::STUCK_RECOVERY_LIMIT_VARIABLE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"FORMAL_AI_STUCK_RECOVERY_SECONDS","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":2629,"end":2712},"sourceWitnesses":[{"path":"rust/src/bounded_autonomy.rs","sha256":
"df8354b549baefbe59a705f480766a09c945e3a3cc2099da9b936d03e6f272d8"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant STUCK_RECOVERY_LIMIT_VARIABLE (exported) (type string) (value (string FORMAL_AI_STUCK_RECOVERY_SECONDS)))",
"maintainedJavaScript":"export const STUCK_RECOVERY_LIMIT_VARIABLE = 'FORMAL_AI_STUCK_RECOVERY_SECONDS';",
"maintainedTranslationSha256":"5958a97551e10aea3bad54fe6bf597a6b36913a2a76e55f034ab755612663a5d"},{"path":
"formal_ai::context_capacity::AVG_UTF8_BYTES_PER_CHAR_ENV","canonicalPath":
"formal_ai::context_capacity::AVG_UTF8_BYTES_PER_CHAR_ENV","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true
,"sourceSpan":{"start":179,"end":261},"sourceWitnesses":[{"path":"rust/src/context_capacity.rs","sha256":
"d6586e4081418b6519ab3163b8d28cd6f4bd9d736606a74afe254fbbbcf24806"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant AVG_UTF8_BYTES_PER_CHAR_ENV (exported) (type string) (value (string FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR)))",
"maintainedJavaScript":"export const AVG_UTF8_BYTES_PER_CHAR_ENV = 'FORMAL_AI_AVG_UTF8_BYTES_PER_CHAR';",
"maintainedTranslationSha256":"cda489b033e3d1f5d1222bdea6b695fb58033e4800fdb546e193f903f7b3ef20"},{"path":
"formal_ai::conversation_context::DIALOG_LOG_DIRECTORY_ENV","canonicalPath":
"formal_ai::conversation_context::DIALOG_LOG_DIRECTORY_ENV","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"FORMAL_AI_DIALOG_LOG_DIR","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":453,"end":523},"sourceWitnesses":[{"path":"rust/src/conversation_context.rs","sha256":
"c4a801de661e08b52ffee1bb818fd2bc9aa003b0e14f8df97f878a87c1d38d91"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DIALOG_LOG_DIRECTORY_ENV (exported) (type string) (value (string FORMAL_AI_DIALOG_LOG_DIR)))",
"maintainedJavaScript":"export const DIALOG_LOG_DIRECTORY_ENV = 'FORMAL_AI_DIALOG_LOG_DIR';",
"maintainedTranslationSha256":"a02493ca206aa9c7555612ff5617ab186397eaf868b50144b56bd81655242dba"},{"path":
"formal_ai::cue_lexicon::CUE_LEXICON_PATH","canonicalPath":"formal_ai::cue_lexicon::CUE_LEXICON_PATH","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"data/meta/cue-lexicon.lino","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":3577,"end":3641},"sourceWitnesses":[{"path":
"rust/src/cue_lexicon.rs","sha256":"a7baaf6eee9179b8511105c1dffa1665a43c13e015612128e9b2ac362f9895c7"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CUE_LEXICON_PATH (exported) (type string) (value (string data/meta/cue-lexicon.lino)))",
"maintainedJavaScript":"export const CUE_LEXICON_PATH = 'data/meta/cue-lexicon.lino';","maintainedTranslationSha256":
"271c05e0ddd54fb2dcc7b6fd756b3fd757ea9822c38f0f7d089c26fd08ef352d"},{"path":"formal_ai::derivation::NOT_RECORDED",
"canonicalPath":"formal_ai::derivation::NOT_RECORDED","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"not recorded","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1838,
"end":1884},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant NOT_RECORDED (exported) (type string) (value (string not%20recorded)))","maintainedJavaScript":
"export const NOT_RECORDED = 'not recorded';","maintainedTranslationSha256":
"4660c96c8f7a81820ce4248ad1190c86aea6a3f3dafc54495bd6b7a9c3a21177"},{"path":"formal_ai::derivation::SEARCH_REQUEST_KIND"
,"canonicalPath":"formal_ai::derivation::SEARCH_REQUEST_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"web_search:request","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":1946,"end":2005},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SEARCH_REQUEST_KIND (exported) (type string) (value (string web_search%3Arequest)))","maintainedJavaScript":
"export const SEARCH_REQUEST_KIND = 'web_search:request';","maintainedTranslationSha256":
"bdc96b34bbada67c36529661b35833ab3a67066eadecddcd8db6af6155e0cb71"},{"path":"formal_ai::derivation::SOURCE_HTTP_KIND",
"canonicalPath":"formal_ai::derivation::SOURCE_HTTP_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"source:http","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start"
:2182,"end":2231},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SOURCE_HTTP_KIND (exported) (type string) (value (string source%3Ahttp)))","maintainedJavaScript":
"export const SOURCE_HTTP_KIND = 'source:http';","maintainedTranslationSha256":
"61a29cc7b107d0f6e9d3c406c0c4e636fee5259ac8ba027de7511d0d30201937"},{"path":
"formal_ai::derivation::FORMALIZE_FRAGMENT_KIND","canonicalPath":"formal_ai::derivation::FORMALIZE_FRAGMENT_KIND","kind"
:"nativeConstant","nativeType":"&str","returnType":"string","value":"formalize:fragment","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2322,"end":2385},"sourceWitnesses":[{"path":
"rust/src/derivation.rs","sha256":"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FORMALIZE_FRAGMENT_KIND (exported) (type string) (value (string formalize%3Afragment)))",
"maintainedJavaScript":"export const FORMALIZE_FRAGMENT_KIND = 'formalize:fragment';","maintainedTranslationSha256":
"02aa178f4c0dea06a912ae138a2db5df8c1ab564bcbdb0ef159fde7eb384cf4e"},{"path":"formal_ai::derivation::DECOMPOSE_PART_KIND"
,"canonicalPath":"formal_ai::derivation::DECOMPOSE_PART_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"decompose:part","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":2493,"end":2548},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DECOMPOSE_PART_KIND (exported) (type string) (value (string decompose%3Apart)))","maintainedJavaScript":
"export const DECOMPOSE_PART_KIND = 'decompose:part';","maintainedTranslationSha256":
"ee410fab98b05069738d78d79a61e0563b1e6ce7b8dd8e5492caec8e37d3ef8c"},{"path":"formal_ai::derivation::RECOMPOSE_BIND_KIND"
,"canonicalPath":"formal_ai::derivation::RECOMPOSE_BIND_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"recompose:bind","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":2694,"end":2749},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant RECOMPOSE_BIND_KIND (exported) (type string) (value (string recompose%3Abind)))","maintainedJavaScript":
"export const RECOMPOSE_BIND_KIND = 'recompose:bind';","maintainedTranslationSha256":
"fa014180b8c62bb72273f0b041450c0b4b3af4aca361ed38797d3d29fdf371ba"},{"path":"formal_ai::derivation::RENDER_EMIT_KIND",
"canonicalPath":"formal_ai::derivation::RENDER_EMIT_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"render:emit","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start"
:2852,"end":2901},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant RENDER_EMIT_KIND (exported) (type string) (value (string render%3Aemit)))","maintainedJavaScript":
"export const RENDER_EMIT_KIND = 'render:emit';","maintainedTranslationSha256":
"1a153a5642ca83fde879488fb95367f49e65daab083213a72adbf9d50646e4ed"},{"path":"formal_ai::derivation::VERIFICATION_KIND",
"canonicalPath":"formal_ai::derivation::VERIFICATION_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"verify:evidence","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":3036,"end":3090},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant VERIFICATION_KIND (exported) (type string) (value (string verify%3Aevidence)))","maintainedJavaScript":
"export const VERIFICATION_KIND = 'verify:evidence';","maintainedTranslationSha256":
"06cbaf6dc469a6aa351170b98d66515e817a3fbb93bcbe72905b647c0a9ba499"},{"path":"formal_ai::derivation::DERIVATIONS_DIR",
"canonicalPath":"formal_ai::derivation::DERIVATIONS_DIR","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"data/cache/derivations","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":4581,"end":4640},"sourceWitnesses":[{"path":"rust/src/derivation.rs","sha256":
"88f3df66b27110966e5f3db1d45b9a7756ba5efe1408ef31c64ca9b56ee36f2f"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DERIVATIONS_DIR (exported) (type string) (value (string data/cache/derivations)))","maintainedJavaScript":
"export const DERIVATIONS_DIR = 'data/cache/derivations';","maintainedTranslationSha256":
"98ce413c2aa2210a551761d3bb906a8a79d1cb2b1d041ae816980b6c6574ae00"},{"path":
"formal_ai::dialog_conversation::CONVERSATION_LOG_SUFFIX","canonicalPath":
"formal_ai::dialog_conversation::CONVERSATION_LOG_SUFFIX","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":".conversation.jsonl","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":
{"start":1080,"end":1144},"sourceWitnesses":[{"path":"rust/src/dialog_conversation.rs","sha256":
"38afb0516b5af9c7f3cb4548f0bcef8e48275f56a76d2b3a6a5a7800375f8092"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CONVERSATION_LOG_SUFFIX (exported) (type string) (value (string .conversation.jsonl)))",
"maintainedJavaScript":"export const CONVERSATION_LOG_SUFFIX = '.conversation.jsonl';","maintainedTranslationSha256":
"22ebeedadc4bacda3e4e495a210dbcf8b3fbf1bd28c44419dd3e0ca5ac00be9d"},{"path":
"formal_ai::document_formats::DOCUMENT_FORMAT_ENGINE","canonicalPath":
"formal_ai::document_formats::DOCUMENT_FORMAT_ENGINE","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"meta_language","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":822,
"end":879},"sourceWitnesses":[{"path":"rust/src/document_formats.rs","sha256":
"2eb43697ce50f3209361d2bf737fc82f5fd8665a422ed337bdbcafeff462d9f0"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DOCUMENT_FORMAT_ENGINE (exported) (type string) (value (string meta_language)))","maintainedJavaScript":
"export const DOCUMENT_FORMAT_ENGINE = 'meta_language';","maintainedTranslationSha256":
"20bddff118f7bdedd73aee0a52711d2c58f0e13fbae2637344b80bfae4227521"},{"path":"formal_ai::DOCUMENT_FORMAT_ENGINE",
"canonicalPath":"formal_ai::document_formats::DOCUMENT_FORMAT_ENGINE","kind":"nativeConstant","nativeType":"&str",
"returnType":"string","value":"meta_language","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":822,"end":879},"sourceWitnesses":[{"path":"rust/src/document_formats.rs","sha256":
"2eb43697ce50f3209361d2bf737fc82f5fd8665a422ed337bdbcafeff462d9f0"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DOCUMENT_FORMAT_ENGINE (exported) (type string) (value (string meta_language)))","maintainedJavaScript":
"export const DOCUMENT_FORMAT_ENGINE = 'meta_language';","maintainedTranslationSha256":
"20bddff118f7bdedd73aee0a52711d2c58f0e13fbae2637344b80bfae4227521"},{"path":
"formal_ai::dreaming_application::STANDING_REQUIREMENT_INTENT","canonicalPath":
"formal_ai::dreaming_application::STANDING_REQUIREMENT_INTENT","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"standing_requirement","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan"
:{"start":6678,"end":6747},"sourceWitnesses":[{"path":"rust/src/dreaming_application.rs","sha256":
"c4eef1bc5485ca20fd72b00d7e36c0c79a8e650662e52adf87d1f9d5bc644152"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant STANDING_REQUIREMENT_INTENT (exported) (type string) (value (string standing_requirement)))",
"maintainedJavaScript":"export const STANDING_REQUIREMENT_INTENT = 'standing_requirement';",
"maintainedTranslationSha256":"ae279e637c82f5402fe4e320fbe139657fb00cd4a92bf537583688bfda01121e"},{"path":
"formal_ai::engine::DEFAULT_MODEL","canonicalPath":"formal_ai::engine::DEFAULT_MODEL","kind":"nativeConstant",
"nativeType":"&str","returnType":"string","value":"formal-ai","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1910,"end":1954},"sourceWitnesses":[{"path":
"rust/src/engine.rs","sha256":"f7bb743802f11c80c6ca039a48025b42fe2d9045a64dcef28d3f7281c87b88c0"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DEFAULT_MODEL (exported) (type string) (value (string formal-ai)))","maintainedJavaScript":
"export const DEFAULT_MODEL = 'formal-ai';","maintainedTranslationSha256":
"ae51fd1ade1b46003293eb348681e0a20e349a9afc0507e08151337e73e0a818"},{"path":"formal_ai::DEFAULT_MODEL","canonicalPath":
"formal_ai::engine::DEFAULT_MODEL","kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"formal-ai"
,"nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1910,"end":1954},
"sourceWitnesses":[{"path":"rust/src/engine.rs","sha256":
"f7bb743802f11c80c6ca039a48025b42fe2d9045a64dcef28d3f7281c87b88c0"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DEFAULT_MODEL (exported) (type string) (value (string formal-ai)))","maintainedJavaScript":
"export const DEFAULT_MODEL = 'formal-ai';","maintainedTranslationSha256":
"ae51fd1ade1b46003293eb348681e0a20e349a9afc0507e08151337e73e0a818"},{"path":
"formal_ai::engine::KNOWLEDGE_SCHEMA_VERSION","canonicalPath":"formal_ai::engine::KNOWLEDGE_SCHEMA_VERSION","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"0.2.0","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":4085,"end":4136},"sourceWitnesses":[{"path":
"rust/src/engine.rs","sha256":"f7bb743802f11c80c6ca039a48025b42fe2d9045a64dcef28d3f7281c87b88c0"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant KNOWLEDGE_SCHEMA_VERSION (exported) (type string) (value (string 0.2.0)))","maintainedJavaScript":
"export const KNOWLEDGE_SCHEMA_VERSION = '0.2.0';","maintainedTranslationSha256":
"0eff15e745b06de01aec565111fa17747e9f30c9daa57ba0672c312eb756053e"},{"path":"formal_ai::execution_box::BACKEND_ENV",
"canonicalPath":"formal_ai::execution_box::BACKEND_ENV","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"FORMAL_AI_EXECUTION_BACKEND","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":1387,"end":1447},"sourceWitnesses":[{"path":"rust/src/execution_box/mod.rs","sha256":
"b8602db59a8956b376ad530c9464a833e63558e4edf764b00991d91d070e2b73"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant BACKEND_ENV (exported) (type string) (value (string FORMAL_AI_EXECUTION_BACKEND)))","maintainedJavaScript":
"export const BACKEND_ENV = 'FORMAL_AI_EXECUTION_BACKEND';","maintainedTranslationSha256":
"84d148ddfa203ea6746d39b5f2337e0436b09f3613fc5dc62614d313e01edfb8"},{"path":
"formal_ai::execution_box::START_ISOLATION_ENV","canonicalPath":"formal_ai::execution_box::START_ISOLATION_ENV","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"FORMAL_AI_START_ISOLATION","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1518,"end":1584},"sourceWitnesses":[{"path":
"rust/src/execution_box/mod.rs","sha256":"b8602db59a8956b376ad530c9464a833e63558e4edf764b00991d91d070e2b73"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant START_ISOLATION_ENV (exported) (type string) (value (string FORMAL_AI_START_ISOLATION)))",
"maintainedJavaScript":"export const START_ISOLATION_ENV = 'FORMAL_AI_START_ISOLATION';","maintainedTranslationSha256":
"70093a513aed2988c7ca84657a325f543c0ffb402624a18a855bd80f62d724b8"},{"path":"formal_ai::execution_box::START_RUNNER_ENV"
,"canonicalPath":"formal_ai::execution_box::START_RUNNER_ENV","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"FORMAL_AI_START_RUNNER","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":1645,"end":1705},"sourceWitnesses":[{"path":"rust/src/execution_box/mod.rs","sha256":
"b8602db59a8956b376ad530c9464a833e63558e4edf764b00991d91d070e2b73"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant START_RUNNER_ENV (exported) (type string) (value (string FORMAL_AI_START_RUNNER)))","maintainedJavaScript":
"export const START_RUNNER_ENV = 'FORMAL_AI_START_RUNNER';","maintainedTranslationSha256":
"f3ed003c9451353d1beca1a7bc966403f4c8b77b23cf135327416e4e7b19cd8f"},{"path":"formal_ai::execution_box::COMMAND_LOG",
"canonicalPath":"formal_ai::execution_box::COMMAND_LOG","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"__formal_ai_commands.log","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":2482,"end":2539},"sourceWitnesses":[{"path":"rust/src/execution_box/mod.rs","sha256":
"b8602db59a8956b376ad530c9464a833e63558e4edf764b00991d91d070e2b73"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant COMMAND_LOG (exported) (type string) (value (string __formal_ai_commands.log)))","maintainedJavaScript":
"export const COMMAND_LOG = '__formal_ai_commands.log';","maintainedTranslationSha256":
"ddbe15b77b43802059327fe13086bc03f9c3dffd212051267cc65902b9a6e686"},{"path":"formal_ai::history_context::SEED_PATH",
"canonicalPath":"formal_ai::history_context::SEED_PATH","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"data/seed/history-formalization.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":
true,"sourceSpan":{"start":1705,"end":1772},"sourceWitnesses":[{"path":"rust/src/history_context.rs","sha256":
"5c25bf3d437bcc38706cfa4df26646c387d28b2fb6125490aebd4fb105a1f6d2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SEED_PATH (exported) (type string) (value (string data/seed/history-formalization.lino)))",
"maintainedJavaScript":"export const SEED_PATH = 'data/seed/history-formalization.lino';","maintainedTranslationSha256":
"890e7add410dfac40a4617740af08d5dc996542d807430f0505d8959c8d058cc"},{"path":"formal_ai::history_context::CURSOR_ROOT",
"canonicalPath":"formal_ai::history_context::CURSOR_ROOT","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"repository_history_cursor","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":1834,"end":1892},"sourceWitnesses":[{"path":"rust/src/history_context.rs","sha256":
"5c25bf3d437bcc38706cfa4df26646c387d28b2fb6125490aebd4fb105a1f6d2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CURSOR_ROOT (exported) (type string) (value (string repository_history_cursor)))","maintainedJavaScript":
"export const CURSOR_ROOT = 'repository_history_cursor';","maintainedTranslationSha256":
"53a96d249e46f8960db69b6bc1cb2db13ee15fc6611f6863bba2aeefaf6f0134"},{"path":
"formal_ai::how_to_capture_manifest::CAPTURE_MANIFEST_FILE","canonicalPath":
"formal_ai::how_to_capture_manifest::CAPTURE_MANIFEST_FILE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"capture-manifest.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":1047,"end":1111},"sourceWitnesses":[{"path":"rust/src/how_to_capture_manifest.rs","sha256":
"a37b18635b09536dc1b3e1e6db817cc66539b55901e9e1b45a7a4cfb6270afbc"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CAPTURE_MANIFEST_FILE (exported) (type string) (value (string capture-manifest.lino)))",
"maintainedJavaScript":"export const CAPTURE_MANIFEST_FILE = 'capture-manifest.lino';","maintainedTranslationSha256":
"57ee23108431c456e6576adede576aad2ecbe394b765723cba7aef24215d28de"},{"path":
"formal_ai::issue_report::SECTION_ENVIRONMENT","canonicalPath":"formal_ai::issue_report::SECTION_ENVIRONMENT","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"## Environment","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1159,"end":1214},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_ENVIRONMENT (exported) (type string) (value (string %23%23%20Environment)))","maintainedJavaScript":
"export const SECTION_ENVIRONMENT = '## Environment';","maintainedTranslationSha256":
"35a6d7efc34fcec5275eaba72c36fd47b688e60cc0eaa04c879a19724e5109d6"},{"path":
"formal_ai::issue_report::SECTION_USER_CONTEXT","canonicalPath":"formal_ai::issue_report::SECTION_USER_CONTEXT","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"## User Context","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1273,"end":1330},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_USER_CONTEXT (exported) (type string) (value (string %23%23%20User%20Context)))",
"maintainedJavaScript":"export const SECTION_USER_CONTEXT = '## User Context';","maintainedTranslationSha256":
"84fb7c6e65e5eb6243ed675f6e5ed9fc0ac0c740e7cbacc7218dc69cb55e66a5"},{"path":
"formal_ai::issue_report::SECTION_REPRODUCTION","canonicalPath":"formal_ai::issue_report::SECTION_REPRODUCTION","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"## Reproduction of dialog","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1370,"end":1437},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_REPRODUCTION (exported) (type string) (value (string %23%23%20Reproduction%20of%20dialog)))",
"maintainedJavaScript":"export const SECTION_REPRODUCTION = '## Reproduction of dialog';","maintainedTranslationSha256":
"0d3ab6f3f8790e5345c66d8b7a62883923ca422cee2ce8674fe754143e7bc3a2"},{"path":
"formal_ai::issue_report::SECTION_REASONING_TRACE","canonicalPath":"formal_ai::issue_report::SECTION_REASONING_TRACE",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"## Reasoning Trace","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1491,"end":1554},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_REASONING_TRACE (exported) (type string) (value (string %23%23%20Reasoning%20Trace)))",
"maintainedJavaScript":"export const SECTION_REASONING_TRACE = '## Reasoning Trace';","maintainedTranslationSha256":
"90b2016ea15f3b0ed05ac3863233814bdce7a72f1a6ac32e4f06508eb10e1bf2"},{"path":
"formal_ai::issue_report::SECTION_DESCRIPTION","canonicalPath":"formal_ai::issue_report::SECTION_DESCRIPTION","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"## Description","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1605,"end":1660},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_DESCRIPTION (exported) (type string) (value (string %23%23%20Description)))","maintainedJavaScript":
"export const SECTION_DESCRIPTION = '## Description';","maintainedTranslationSha256":
"85a412a6966cc780f2d0f513411c8cf42efa5d6b4d9b2823c3812760889240e4"},{"path":
"formal_ai::issue_report::SECTION_ATTACH_MEMORY","canonicalPath":"formal_ai::issue_report::SECTION_ATTACH_MEMORY","kind"
:"nativeConstant","nativeType":"&str","returnType":"string","value":"## Attach full memory (optional)",
"nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1715,"end":1790},
"sourceWitnesses":[{"path":"rust/src/issue_report.rs","sha256":
"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SECTION_ATTACH_MEMORY (exported) (type string) (value (string %23%23%20Attach%20full%20memory%20%28optional%29)))"
,"maintainedJavaScript":"export const SECTION_ATTACH_MEMORY = '## Attach full memory (optional)';",
"maintainedTranslationSha256":"d195d457097ba82ccc44e7173ba5b97ac7dfac686f6856d6de79644cb1dad37d"},{"path":
"formal_ai::issue_report::COUNT_PLACEHOLDER","canonicalPath":"formal_ai::issue_report::COUNT_PLACEHOLDER","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"{count}","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2121,"end":2167},"sourceWitnesses":[{"path":
"rust/src/issue_report.rs","sha256":"b6b9860eecbe63644ae55455eab45b62cda0248adc7348d60fb2ae753915a856"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant COUNT_PLACEHOLDER (exported) (type string) (value (string %7Bcount%7D)))","maintainedJavaScript":
"export const COUNT_PLACEHOLDER = '{count}';","maintainedTranslationSha256":
"8d563257788b45e7caf53f86cbcafcf7ccc54746308a58916e20d7f5f775abb5"},{"path":
"formal_ai::language_adoption::LANGUAGE_GAP_FRONTIER","canonicalPath":
"formal_ai::language_adoption::LANGUAGE_GAP_FRONTIER","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"language-gap","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1625,
"end":1680},"sourceWitnesses":[{"path":"rust/src/language_adoption.rs","sha256":
"feba773ffd1856dd78335022d6c32612a60beb45bd17b7d14b2209969dc21853"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LANGUAGE_GAP_FRONTIER (exported) (type string) (value (string language-gap)))","maintainedJavaScript":
"export const LANGUAGE_GAP_FRONTIER = 'language-gap';","maintainedTranslationSha256":
"325409b8254dc6222602b497da51513123965d7620d3b6b58eb562c6fb8ee62b"},{"path":
"formal_ai::learning_cycle::GOOGLE_TRENDS_FRONTIER","canonicalPath":"formal_ai::learning_cycle::GOOGLE_TRENDS_FRONTIER",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"google-trends","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":3226,"end":3283},"sourceWitnesses":[{"path":
"rust/src/learning_cycle.rs","sha256":"0a173a3bb4f7eabe8a2ff385c66afbc92d6d93c77772d180a7fe6ced5ee118a1"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant GOOGLE_TRENDS_FRONTIER (exported) (type string) (value (string google-trends)))","maintainedJavaScript":
"export const GOOGLE_TRENDS_FRONTIER = 'google-trends';","maintainedTranslationSha256":
"7c8a09632b38fea97d92a6ed4ca2459484e91ab703492b3297d1abf173cf6f8b"},{"path":
"formal_ai::learning_cycle::LANGUAGE_GAP_FRONTIER","canonicalPath":"formal_ai::learning_cycle::LANGUAGE_GAP_FRONTIER",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"language-gap","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":3548,"end":3603},"sourceWitnesses":[{"path":
"rust/src/learning_cycle.rs","sha256":"0a173a3bb4f7eabe8a2ff385c66afbc92d6d93c77772d180a7fe6ced5ee118a1"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LANGUAGE_GAP_FRONTIER (exported) (type string) (value (string language-gap)))","maintainedJavaScript":
"export const LANGUAGE_GAP_FRONTIER = 'language-gap';","maintainedTranslationSha256":
"325409b8254dc6222602b497da51513123965d7620d3b6b58eb562c6fb8ee62b"},{"path":
"formal_ai::learning_cycle::UPSTREAM_BENCHMARKS_FRONTIER","canonicalPath":
"formal_ai::learning_cycle::UPSTREAM_BENCHMARKS_FRONTIER","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"upstream-benchmarks","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":
{"start":3927,"end":3996},"sourceWitnesses":[{"path":"rust/src/learning_cycle.rs","sha256":
"0a173a3bb4f7eabe8a2ff385c66afbc92d6d93c77772d180a7fe6ced5ee118a1"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant UPSTREAM_BENCHMARKS_FRONTIER (exported) (type string) (value (string upstream-benchmarks)))",
"maintainedJavaScript":"export const UPSTREAM_BENCHMARKS_FRONTIER = 'upstream-benchmarks';",
"maintainedTranslationSha256":"ed61f3629080302ea283454b85467b97f5222f2b98e7f806a70145c753997457"},{"path":
"formal_ai::learning_cycle::LEARNED_REQUEST_OPENERS_SEED_FILE","canonicalPath":
"formal_ai::learning_cycle::LEARNED_REQUEST_OPENERS_SEED_FILE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"data/seed/learned-request-openers.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified"
:true,"sourceSpan":{"start":7607,"end":7700},"sourceWitnesses":[{"path":"rust/src/learning_cycle.rs","sha256":
"0a173a3bb4f7eabe8a2ff385c66afbc92d6d93c77772d180a7fe6ced5ee118a1"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LEARNED_REQUEST_OPENERS_SEED_FILE (exported) (type string) (value (string data/seed/learned-request-openers.lino)))"
,"maintainedJavaScript":"export const LEARNED_REQUEST_OPENERS_SEED_FILE = 'data/seed/learned-request-openers.lino';",
"maintainedTranslationSha256":"047ff29a5124216a2ed0c9c89da15947eead13387cf7b39c2eb60d5eb397588b"},{"path":
"formal_ai::learning_cycle::TERM_INFORMATION_ROLE","canonicalPath":"formal_ai::learning_cycle::TERM_INFORMATION_ROLE",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"term_information_request_opener",
"nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":7875,"end":7949},
"sourceWitnesses":[{"path":"rust/src/learning_cycle.rs","sha256":
"0a173a3bb4f7eabe8a2ff385c66afbc92d6d93c77772d180a7fe6ced5ee118a1"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant TERM_INFORMATION_ROLE (exported) (type string) (value (string term_information_request_opener)))",
"maintainedJavaScript":"export const TERM_INFORMATION_ROLE = 'term_information_request_opener';",
"maintainedTranslationSha256":"b435e9cb91af878f5d83a95bcd22c60e1f7a91386272c6874b3f57bdfd1064f0"},{"path":
"formal_ai::lexeme_import::PART_OF_SPEECH","canonicalPath":"formal_ai::lexeme_import::PART_OF_SPEECH","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"noun","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":4046,"end":4086},"sourceWitnesses":[{"path":
"rust/src/lexeme_import.rs","sha256":"b4c4dabb9fff00d550349a2a01115adca957271e036f623041f183b08153ba56"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant PART_OF_SPEECH (exported) (type string) (value (string noun)))","maintainedJavaScript":
"export const PART_OF_SPEECH = 'noun';","maintainedTranslationSha256":
"4ed83094b8148f992f945d65f94ec601d4969c5aac36676496b9ce722ddcb6ef"},{"path":
"formal_ai::lexeme_import::GRAMMATICAL_NUMBER","canonicalPath":"formal_ai::lexeme_import::GRAMMATICAL_NUMBER","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"singular","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":4169,"end":4217},"sourceWitnesses":[{"path":
"rust/src/lexeme_import.rs","sha256":"b4c4dabb9fff00d550349a2a01115adca957271e036f623041f183b08153ba56"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant GRAMMATICAL_NUMBER (exported) (type string) (value (string singular)))","maintainedJavaScript":
"export const GRAMMATICAL_NUMBER = 'singular';","maintainedTranslationSha256":
"8ed21761fd68bfb4d1255ed07d1e46a88a42ec2bf035efc99485484d7db76868"},{"path":"formal_ai::lexeme_import::DEFINED_BY",
"canonicalPath":"formal_ai::lexeme_import::DEFINED_BY","kind":"nativeConstant","nativeType":"&str","returnType":"string"
,"value":"entity","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":4271,"end":
4309},"sourceWitnesses":[{"path":"rust/src/lexeme_import.rs","sha256":
"b4c4dabb9fff00d550349a2a01115adca957271e036f623041f183b08153ba56"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DEFINED_BY (exported) (type string) (value (string entity)))","maintainedJavaScript":
"export const DEFINED_BY = 'entity';","maintainedTranslationSha256":
"f3a4ae2781bed15bcb7b97a2887c7a8066dcdd503902d95f540236b5ba803a9e"},{"path":"formal_ai::memory_sync::LEARNED_CHUNK_KIND"
,"canonicalPath":"formal_ai::memory_sync::LEARNED_CHUNK_KIND","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"meta_learned_chunk","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":7336,"end":7394},"sourceWitnesses":[{"path":"rust/src/memory_sync.rs","sha256":
"9ba99a84a01c29e5466b63d52b07d9398e9043d8fa6630e9f9e1429ddb795b91"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LEARNED_CHUNK_KIND (exported) (type string) (value (string meta_learned_chunk)))","maintainedJavaScript":
"export const LEARNED_CHUNK_KIND = 'meta_learned_chunk';","maintainedTranslationSha256":
"0583264644e747046295be38faf1c483d438406f7fce54374131fdfa98467787"},{"path":
"formal_ai::method_learning::LEARNED_METHODS_SEED_FILE","canonicalPath":
"formal_ai::method_learning::LEARNED_METHODS_SEED_FILE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"data/seed/learned-methods.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":818,"end":895},"sourceWitnesses":[{"path":"rust/src/method_learning.rs","sha256":
"cead5763f6305f66d5fe1d8982817803b59a24fe0735be417c5b676b0b003ac3"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LEARNED_METHODS_SEED_FILE (exported) (type string) (value (string data/seed/learned-methods.lino)))",
"maintainedJavaScript":"export const LEARNED_METHODS_SEED_FILE = 'data/seed/learned-methods.lino';",
"maintainedTranslationSha256":"1ee19d98f6e52a81c7579540efadb4de2912a883c63ad2d0292440439bbff8ec"},{"path":
"formal_ai::program_plan::TASK_NODE","canonicalPath":"formal_ai::program_plan::TASK_NODE","kind":"nativeConstant",
"nativeType":"&str","returnType":"string","value":"request:task","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2170,"end":2213},"sourceWitnesses":[{"path":
"rust/src/program_plan.rs","sha256":"006f093b706dae68c00b471262677bb14276a490d7ccd3b7a92c11024695341a"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant TASK_NODE (exported) (type string) (value (string request%3Atask)))","maintainedJavaScript":
"export const TASK_NODE = 'request:task';","maintainedTranslationSha256":
"fd7d3540dff271cd9de5695a66534ee83cccf1a6a6d31fa60ab5910c5fae786e"},{"path":"formal_ai::program_plan::MODIFIER_NODE",
"canonicalPath":"formal_ai::program_plan::MODIFIER_NODE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"request:modifier","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":2272,"end":2323},"sourceWitnesses":[{"path":"rust/src/program_plan.rs","sha256":
"006f093b706dae68c00b471262677bb14276a490d7ccd3b7a92c11024695341a"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MODIFIER_NODE (exported) (type string) (value (string request%3Amodifier)))","maintainedJavaScript":
"export const MODIFIER_NODE = 'request:modifier';","maintainedTranslationSha256":
"28fc7132cb69641fb41f6786508f7a95717305019adbf8747d686d9c95d5c623"},{"path":
"formal_ai::program_skill_gap::MISSING_PARAMETER","canonicalPath":"formal_ai::program_skill_gap::MISSING_PARAMETER",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"missing","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2552,"end":2598},"sourceWitnesses":[{"path":
"rust/src/program_skill_gap.rs","sha256":"ea02d17245c30a61a183ad058bb83121c03805c266a720f1f045490a5a1b63df"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MISSING_PARAMETER (exported) (type string) (value (string missing)))","maintainedJavaScript":
"export const MISSING_PARAMETER = 'missing';","maintainedTranslationSha256":
"074c3f9643e71c0b2fb0fed9c13ee6bc10789eb1a2117280f77588faa127b552"},{"path":
"formal_ai::promotion::LEARNED_PROGRAM_RULES_SEED_FILE","canonicalPath":
"formal_ai::promotion::LEARNED_PROGRAM_RULES_SEED_FILE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"data/seed/learned-program-rules.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":
true,"sourceSpan":{"start":2323,"end":2412},"sourceWitnesses":[{"path":"rust/src/promotion.rs","sha256":
"9409722910d6be666afc48b9fe9db222e416e4227fab7366ac0a8296826baacb"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LEARNED_PROGRAM_RULES_SEED_FILE (exported) (type string) (value (string data/seed/learned-program-rules.lino)))"
,"maintainedJavaScript":"export const LEARNED_PROGRAM_RULES_SEED_FILE = 'data/seed/learned-program-rules.lino';",
"maintainedTranslationSha256":"48aaf1bae8888eaa76f9ec154fb3e5992d2f9b22ce6eb4c08fab0a847249b931"},{"path":
"formal_ai::LEARNED_PROGRAM_RULES_SEED_FILE","canonicalPath":"formal_ai::promotion::LEARNED_PROGRAM_RULES_SEED_FILE",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"data/seed/learned-program-rules.lino",
"nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2323,"end":2412},
"sourceWitnesses":[{"path":"rust/src/promotion.rs","sha256":
"9409722910d6be666afc48b9fe9db222e416e4227fab7366ac0a8296826baacb"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant LEARNED_PROGRAM_RULES_SEED_FILE (exported) (type string) (value (string data/seed/learned-program-rules.lino)))"
,"maintainedJavaScript":"export const LEARNED_PROGRAM_RULES_SEED_FILE = 'data/seed/learned-program-rules.lino';",
"maintainedTranslationSha256":"48aaf1bae8888eaa76f9ec154fb3e5992d2f9b22ce6eb4c08fab0a847249b931"},{"path":
"formal_ai::rust_projection::REFUSAL_LANGUAGE","canonicalPath":"formal_ai::rust_projection::REFUSAL_LANGUAGE","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"grammar-projection-refusal","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1257,"end":1321},"sourceWitnesses":[{"path":
"rust/src/rust_projection.rs","sha256":"bdb8886a1f657db4d4456302efdd66fc57e498a8a906f62bc570ba120c1bc4bc"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant REFUSAL_LANGUAGE (exported) (type string) (value (string grammar-projection-refusal)))",
"maintainedJavaScript":"export const REFUSAL_LANGUAGE = 'grammar-projection-refusal';","maintainedTranslationSha256":
"0e9084f7772a99aeba88ff6e3b2f57d490ec78c66cea91397cc6edb11b005a3b"},{"path":
"formal_ai::rust_projection::REFUSAL_NOFORM_LANGUAGE","canonicalPath":
"formal_ai::rust_projection::REFUSAL_NOFORM_LANGUAGE","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"grammar-projection-noform","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":1533,"end":1603},"sourceWitnesses":[{"path":"rust/src/rust_projection.rs","sha256":
"bdb8886a1f657db4d4456302efdd66fc57e498a8a906f62bc570ba120c1bc4bc"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant REFUSAL_NOFORM_LANGUAGE (exported) (type string) (value (string grammar-projection-noform)))",
"maintainedJavaScript":"export const REFUSAL_NOFORM_LANGUAGE = 'grammar-projection-noform';",
"maintainedTranslationSha256":"5443ae9eff34bee810d473038310d14834cd954e16d3059d29f2a7248f1977d1"},{"path":
"formal_ai::rust_projection::ANY_TARGET","canonicalPath":"formal_ai::rust_projection::ANY_TARGET","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"any","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1676,"end":1711},"sourceWitnesses":[{"path":
"rust/src/rust_projection.rs","sha256":"bdb8886a1f657db4d4456302efdd66fc57e498a8a906f62bc570ba120c1bc4bc"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant ANY_TARGET (exported) (type string) (value (string any)))","maintainedJavaScript":
"export const ANY_TARGET = 'any';","maintainedTranslationSha256":
"ae4d711c22272988156537c51508d277a655d088ef427e630ab1fb53a70c007c"},{"path":
"formal_ai::search_fusion_learning::SEARCH_FUSION_TASK_FAMILY","canonicalPath":
"formal_ai::search_fusion_learning::SEARCH_FUSION_TASK_FAMILY","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"captured_search_statement_fusion","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":937,"end":1016},"sourceWitnesses":[{"path":"rust/src/search_fusion_learning.rs","sha256":
"d44dd9dedb1d559f0e271b40aa3c8bbad64c583381c05c8abb927da3c8d18053"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SEARCH_FUSION_TASK_FAMILY (exported) (type string) (value (string captured_search_statement_fusion)))",
"maintainedJavaScript":"export const SEARCH_FUSION_TASK_FAMILY = 'captured_search_statement_fusion';",
"maintainedTranslationSha256":"7916d1e770c39497efd1669f005b3bdb4e4710b53add3ed83b90e100f76cd839"},{"path":
"formal_ai::SEARCH_FUSION_TASK_FAMILY","canonicalPath":"formal_ai::search_fusion_learning::SEARCH_FUSION_TASK_FAMILY",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"captured_search_statement_fusion",
"nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":937,"end":1016},
"sourceWitnesses":[{"path":"rust/src/search_fusion_learning.rs","sha256":
"d44dd9dedb1d559f0e271b40aa3c8bbad64c583381c05c8abb927da3c8d18053"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SEARCH_FUSION_TASK_FAMILY (exported) (type string) (value (string captured_search_statement_fusion)))",
"maintainedJavaScript":"export const SEARCH_FUSION_TASK_FAMILY = 'captured_search_statement_fusion';",
"maintainedTranslationSha256":"7916d1e770c39497efd1669f005b3bdb4e4710b53add3ed83b90e100f76cd839"},{"path":
"formal_ai::self_ast_census::CENSUS_DIR","canonicalPath":"formal_ai::self_ast_census::CENSUS_DIR","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"data/meta/self-ast","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2048,"end":2098},"sourceWitnesses":[{"path":
"rust/src/self_ast_census.rs","sha256":"aa376c1b4fd9efa57cffc2716574e377bfdc9e4ed7273bed70a2a3b4154016aa"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CENSUS_DIR (exported) (type string) (value (string data/meta/self-ast)))","maintainedJavaScript":
"export const CENSUS_DIR = 'data/meta/self-ast';","maintainedTranslationSha256":
"5bacb60f3e9d6d6cab9b6499959ea693498f51cf0f8266be55e3fe9cfdcc6c7e"},{"path":
"formal_ai::self_ast_census::FULL_FIDELITY_PREFIX","canonicalPath":"formal_ai::self_ast_census::FULL_FIDELITY_PREFIX",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":"src/agentic_coding/","nativeSourceQualified":
true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2367,"end":2428},"sourceWitnesses":[{"path":
"rust/src/self_ast_census.rs","sha256":"aa376c1b4fd9efa57cffc2716574e377bfdc9e4ed7273bed70a2a3b4154016aa"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FULL_FIDELITY_PREFIX (exported) (type string) (value (string src/agentic_coding/)))","maintainedJavaScript":
"export const FULL_FIDELITY_PREFIX = 'src/agentic_coding/';","maintainedTranslationSha256":
"f46bdc8e27a78efe21eddfc2e9e417085ba96a7e5c3aa457d318dbc7253c875e"},{"path":
"formal_ai::self_improvement::NO_GATE_EVIDENCE","canonicalPath":"formal_ai::self_improvement::NO_GATE_EVIDENCE","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"no_gate_evidence","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":5098,"end":5152},"sourceWitnesses":[{"path":
"rust/src/self_improvement.rs","sha256":"82623d0187bf5b0917a5f809185d5b3a6f101d7bb58f66ccb56fa26703a96eef"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant NO_GATE_EVIDENCE (exported) (type string) (value (string no_gate_evidence)))","maintainedJavaScript":
"export const NO_GATE_EVIDENCE = 'no_gate_evidence';","maintainedTranslationSha256":
"de26fe327f61a7e3ef2469bba68561a32e1c35ed4d0b743b03e5ac21bc697b18"},{"path":
"formal_ai::service_accessibility::SERVICE_ACCESSIBILITY_FILE","canonicalPath":
"formal_ai::service_accessibility::SERVICE_ACCESSIBILITY_FILE","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"service-accessibility.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,
"sourceSpan":{"start":2158,"end":2232},"sourceWitnesses":[{"path":"rust/src/service_accessibility.rs","sha256":
"73c8f538c1d2cd1bfabc630a4000dd823344c4396a6a5b46f4d3cfcd24e36bc2"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SERVICE_ACCESSIBILITY_FILE (exported) (type string) (value (string service-accessibility.lino)))",
"maintainedJavaScript":"export const SERVICE_ACCESSIBILITY_FILE = 'service-accessibility.lino';",
"maintainedTranslationSha256":"bec637345269b596949b9228b84bb1232c294bf240c79ace511b736d960d845a"},{"path":
"formal_ai::shared_memory::MEMORY_PATH_ENV","canonicalPath":"formal_ai::shared_memory::MEMORY_PATH_ENV","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"FORMAL_AI_MEMORY_PATH","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":93,"end":151},"sourceWitnesses":[{"path":
"rust/src/shared_memory.rs","sha256":"093df7e7d711488950900841ddbe826b01467a96fdb7bf633c13af7d81340cfc"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MEMORY_PATH_ENV (exported) (type string) (value (string FORMAL_AI_MEMORY_PATH)))","maintainedJavaScript":
"export const MEMORY_PATH_ENV = 'FORMAL_AI_MEMORY_PATH';","maintainedTranslationSha256":
"85a67332fa42fe15a2492d515349f835cfc24f032db7c262021d79b1671ef70c"},{"path":"formal_ai::MEMORY_PATH_ENV","canonicalPath"
:"formal_ai::shared_memory::MEMORY_PATH_ENV","kind":"nativeConstant","nativeType":"&str","returnType":"string","value":
"FORMAL_AI_MEMORY_PATH","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":93,
"end":151},"sourceWitnesses":[{"path":"rust/src/shared_memory.rs","sha256":
"093df7e7d711488950900841ddbe826b01467a96fdb7bf633c13af7d81340cfc"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MEMORY_PATH_ENV (exported) (type string) (value (string FORMAL_AI_MEMORY_PATH)))","maintainedJavaScript":
"export const MEMORY_PATH_ENV = 'FORMAL_AI_MEMORY_PATH';","maintainedTranslationSha256":
"85a67332fa42fe15a2492d515349f835cfc24f032db7c262021d79b1671ef70c"},{"path":
"formal_ai::shared_memory::MEMORY_DIRECTORY_NAME","canonicalPath":"formal_ai::shared_memory::MEMORY_DIRECTORY_NAME",
"kind":"nativeConstant","nativeType":"&str","returnType":"string","value":".formal-ai","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":152,"end":205},"sourceWitnesses":[{"path":
"rust/src/shared_memory.rs","sha256":"093df7e7d711488950900841ddbe826b01467a96fdb7bf633c13af7d81340cfc"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MEMORY_DIRECTORY_NAME (exported) (type string) (value (string .formal-ai)))","maintainedJavaScript":
"export const MEMORY_DIRECTORY_NAME = '.formal-ai';","maintainedTranslationSha256":
"64dd553dd98d54408a6eafbe00fdc0f534c50264023b7748099bb6fc17c71e64"},{"path":"formal_ai::shared_memory::MEMORY_FILE_NAME"
,"canonicalPath":"formal_ai::shared_memory::MEMORY_FILE_NAME","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"memory.lino","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start"
:206,"end":255},"sourceWitnesses":[{"path":"rust/src/shared_memory.rs","sha256":
"093df7e7d711488950900841ddbe826b01467a96fdb7bf633c13af7d81340cfc"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant MEMORY_FILE_NAME (exported) (type string) (value (string memory.lino)))","maintainedJavaScript":
"export const MEMORY_FILE_NAME = 'memory.lino';","maintainedTranslationSha256":
"bd141d2fd7331d9a4020a1d4deab16a429c1f442980b7ff087c97fcfb0b5a013"},{"path":"formal_ai::si_units::ENGINE_NAME",
"canonicalPath":"formal_ai::si_units::ENGINE_NAME","kind":"nativeConstant","nativeType":"&str","returnType":"string",
"value":"si-dimension-algebra","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{"start"
:2033,"end":2086},"sourceWitnesses":[{"path":"rust/src/si_units.rs","sha256":
"afea94b3dce2ee6f38a229b960d9949ccd59a508e463aa6ec92ff48a75cfe430"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant ENGINE_NAME (exported) (type string) (value (string si-dimension-algebra)))","maintainedJavaScript":
"export const ENGINE_NAME = 'si-dimension-algebra';","maintainedTranslationSha256":
"c351e62c77e2839d233c70a873476f3d7734bedd2d4a1015246bf90b4473d006"},{"path":
"formal_ai::thinking::DEFAULT_THINKING_LANGUAGE","canonicalPath":"formal_ai::thinking::DEFAULT_THINKING_LANGUAGE","kind"
:"nativeConstant","nativeType":"&str","returnType":"string","value":"en","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":1614,"end":1663},"sourceWitnesses":[{"path":
"rust/src/thinking.rs","sha256":"bce5adb875301e2cd98155c354ec06aa2a1f8f09fc0f15d1fb9ce13df333f766"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant DEFAULT_THINKING_LANGUAGE (exported) (type string) (value (string en)))","maintainedJavaScript":
"export const DEFAULT_THINKING_LANGUAGE = 'en';","maintainedTranslationSha256":
"b7c7a37bd97df21fb69061c22b9e3e14059a102df092084eb5c8085e467844c0"},{"path":"formal_ai::world_model_atoms::TARGET_CUES",
"canonicalPath":"formal_ai::world_model_atoms::TARGET_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_target","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":1944,"end":1995},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant TARGET_CUES (exported) (type string) (value (string world_state_target)))","maintainedJavaScript":
"export const TARGET_CUES = 'world_state_target';","maintainedTranslationSha256":
"2d4647bcf0ba0724257e1e224ffbb74da03030c6548709f0e9d8a8195ba87c4f"},{"path":"formal_ai::world_model_atoms::QUERY_CUES",
"canonicalPath":"formal_ai::world_model_atoms::QUERY_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_query","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":2056,"end":2105},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant QUERY_CUES (exported) (type string) (value (string world_state_query)))","maintainedJavaScript":
"export const QUERY_CUES = 'world_state_query';","maintainedTranslationSha256":
"22fe1f17c8b904a081e77c8c7798c3f3eeb16f967fd79a44294188df9cdfd78d"},{"path":"formal_ai::world_model_atoms::CONFIRM_CUES"
,"canonicalPath":"formal_ai::world_model_atoms::CONFIRM_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_confirm","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":
{"start":2179,"end":2232},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CONFIRM_CUES (exported) (type string) (value (string world_state_confirm)))","maintainedJavaScript":
"export const CONFIRM_CUES = 'world_state_confirm';","maintainedTranslationSha256":
"3e2c991a4f94533c88ec5f51e3204b7f19df9d0246c8aee017196fe6e7d65376"},{"path":"formal_ai::world_model_atoms::CORRECT_CUES"
,"canonicalPath":"formal_ai::world_model_atoms::CORRECT_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_correct","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":
{"start":2307,"end":2360},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant CORRECT_CUES (exported) (type string) (value (string world_state_correct)))","maintainedJavaScript":
"export const CORRECT_CUES = 'world_state_correct';","maintainedTranslationSha256":
"aaed6e18d5dc956261eba10208d6d1586449822b3e86564fd3bd8427f52b0da7"},{"path":"formal_ai::world_model_atoms::BECAUSE_CUES"
,"canonicalPath":"formal_ai::world_model_atoms::BECAUSE_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_because","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":
{"start":2493,"end":2546},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant BECAUSE_CUES (exported) (type string) (value (string world_state_because)))","maintainedJavaScript":
"export const BECAUSE_CUES = 'world_state_because';","maintainedTranslationSha256":
"9b5e368f505c7e18da01b3159c1615dd311e69e7f9dbcdd09fae7b2062c0f2d2"},{"path":
"formal_ai::world_model_atoms::SEPARATOR_CUES","canonicalPath":"formal_ai::world_model_atoms::SEPARATOR_CUES","kind":
"nativeConstant","nativeType":"&str","returnType":"string","value":"world_state_separator","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":2618,"end":2675},"sourceWitnesses":[{"path":
"rust/src/world_model_atoms.rs","sha256":"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SEPARATOR_CUES (exported) (type string) (value (string world_state_separator)))","maintainedJavaScript":
"export const SEPARATOR_CUES = 'world_state_separator';","maintainedTranslationSha256":
"310df6496fb40f310a10ea6aec4b9d630f07f4bd8c0a741156ec045227969c4e"},{"path":"formal_ai::world_model_atoms::FILLER_CUES",
"canonicalPath":"formal_ai::world_model_atoms::FILLER_CUES","kind":"nativeConstant","nativeType":"&str","returnType":
"string","value":"world_state_filler","nativeSourceQualified":true,"nativeBorrowedConstantQualified":true,"sourceSpan":{
"start":2752,"end":2803},"sourceWitnesses":[{"path":"rust/src/world_model_atoms.rs","sha256":
"9b21e9ca8a9789e0a2f08e1de517a3afccb4bdc49d1e89f39c23fafb2068eb09"},{"path":"rust/src/lib.rs","sha256":
"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant FILLER_CUES (exported) (type string) (value (string world_state_filler)))","maintainedJavaScript":
"export const FILLER_CUES = 'world_state_filler';","maintainedTranslationSha256":
"82f49318ed03be279ccd2393f1548b6a59216fbba9beb82093194278df66f3bb"},{"path":
"formal_ai::world_model_dialog::SYNC_CHAIN_ROOT","canonicalPath":"formal_ai::world_model_dialog::SYNC_CHAIN_ROOT","kind"
:"nativeConstant","nativeType":"&str","returnType":"string","value":"world_sync_root","nativeSourceQualified":true,
"nativeBorrowedConstantQualified":true,"sourceSpan":{"start":7412,"end":7464},"sourceWitnesses":[{"path":
"rust/src/world_model_dialog.rs","sha256":"4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"},{"path":
"rust/src/lib.rs","sha256":"a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"}],"maintainedMeta":
"(constant SYNC_CHAIN_ROOT (exported) (type string) (value (string world_sync_root)))","maintainedJavaScript":
"export const SYNC_CHAIN_ROOT = 'world_sync_root';","maintainedTranslationSha256":
"d92ef7d5d2296fa344b5fe96159bd65f390548dbf78fcbf572febbebd0302214"}];
