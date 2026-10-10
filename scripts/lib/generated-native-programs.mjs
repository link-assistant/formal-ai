// Generated source-qualified substitution programs. Unknown algorithms remain refused.
export const GENERATED_NATIVE_PROGRAMS = [
  {
    "path": "formal_ai::summarization::apply_compound_words",
    "kind": "substitutionProgram",
    "signature": [
      "string",
      "string"
    ],
    "returnType": "string",
    "program": {
      "selectorArgument": 1,
      "inputArgument": 0,
      "alternatives": [
        {
          "selector": "ru",
          "pairs": [
            [
              "в которой ",
              "где "
            ],
            [
              "для того чтобы ",
              "чтобы "
            ],
            [
              "к примеру",
              "например"
            ]
          ]
        },
        {
          "fallback": true,
          "pairs": [
            [
              "in order to ",
              "to "
            ],
            [
              "for the purpose of ",
              "for "
            ],
            [
              "a number of ",
              "several "
            ],
            [
              "user interface",
              "UI"
            ],
            [
              "command line interface",
              "CLI"
            ],
            [
              "artificial intelligence",
              "AI"
            ]
          ]
        }
      ]
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::apply_semantic_primes",
    "kind": "substitutionProgram",
    "signature": [
      "string",
      "string"
    ],
    "returnType": "string",
    "program": {
      "selectorArgument": 1,
      "inputArgument": 0,
      "alternatives": [
        {
          "selector": "ru",
          "pairs": [
            [
              "автоматизация",
              "когда машина делает"
            ],
            [
              "оркестрирует",
              "управляет вместе"
            ],
            [
              "делегирование",
              "передача работы"
            ],
            [
              "детерминированный",
              "всегда одинаковый"
            ]
          ]
        },
        {
          "fallback": true,
          "pairs": [
            [
              "orchestrates",
              "controls many"
            ],
            [
              "automation of automation",
              "machine that makes other machines do"
            ],
            [
              "automation",
              "machine doing"
            ],
            [
              "delegating",
              "giving work to"
            ],
            [
              "deterministic",
              "always the same"
            ],
            [
              "multilingual",
              "in many languages"
            ],
            [
              "symbolic",
              "rule-based"
            ]
          ]
        }
      ]
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::DEFAULT_MAX_STATEMENTS",
    "kind": "nativeConstant",
    "nativeType": "usize",
    "value": 30,
    "returnType": "number",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3100,
      "end": 3145
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Misc",
    "signature": [],
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Identity",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Identity",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Purpose",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Purpose",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Language",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Language",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Stars",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Stars",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Feature",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Feature",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::UseCase",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "UseCase",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Install",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Install",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Example",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Example",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::Misc",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::StatementKind",
    "variant": "Misc",
    "returnType": "enum:formal_ai::summarization::StatementKind",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3329,
      "end": 4008
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Standard",
    "signature": [],
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Identifier",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Identifier",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Topic",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Topic",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Short",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Short",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Standard",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Standard",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Full",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Full",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::Expand",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "variant": "Expand",
    "returnType": "enum:formal_ai::summarization::SummarizationMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 7123,
      "end": 8002
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::is_essential",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::summarization::StatementKind",
    "signature": [
      "enum:formal_ai::summarization::StatementKind"
    ],
    "name": "is_essential",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Identity",
        "value": true
      },
      {
        "variant": "Purpose",
        "value": true
      },
      {
        "variant": "Language",
        "value": true
      },
      {
        "variant": "Stars",
        "value": true
      },
      {
        "variant": "Feature",
        "value": false
      },
      {
        "variant": "UseCase",
        "value": false
      },
      {
        "variant": "Install",
        "value": false
      },
      {
        "variant": "Example",
        "value": false
      },
      {
        "variant": "Misc",
        "value": false
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 4010,
      "end": 6227
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::StatementKind::is_boilerplate",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::summarization::StatementKind",
    "signature": [
      "enum:formal_ai::summarization::StatementKind"
    ],
    "name": "is_boilerplate",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Identity",
        "value": false
      },
      {
        "variant": "Purpose",
        "value": false
      },
      {
        "variant": "Language",
        "value": false
      },
      {
        "variant": "Stars",
        "value": false
      },
      {
        "variant": "Feature",
        "value": false
      },
      {
        "variant": "UseCase",
        "value": false
      },
      {
        "variant": "Install",
        "value": true
      },
      {
        "variant": "Example",
        "value": true
      },
      {
        "variant": "Misc",
        "value": false
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 4010,
      "end": 6227
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::target_percent",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "signature": [
      "enum:formal_ai::summarization::SummarizationMode"
    ],
    "name": "target_percent",
    "nativeType": "u32",
    "returnType": "number",
    "cases": [
      {
        "variant": "Identifier",
        "value": 0
      },
      {
        "variant": "Topic",
        "value": 0
      },
      {
        "variant": "Short",
        "value": 20
      },
      {
        "variant": "Standard",
        "value": 50
      },
      {
        "variant": "Full",
        "value": 100
      },
      {
        "variant": "Expand",
        "value": 200
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 8004,
      "end": 9742
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::summarization::SummarizationMode::is_label_only",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::summarization::SummarizationMode",
    "signature": [
      "enum:formal_ai::summarization::SummarizationMode"
    ],
    "name": "is_label_only",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Identifier",
        "value": true
      },
      {
        "variant": "Topic",
        "value": true
      },
      {
        "variant": "Short",
        "value": false
      },
      {
        "variant": "Standard",
        "value": false
      },
      {
        "variant": "Full",
        "value": false
      },
      {
        "variant": "Expand",
        "value": false
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 8004,
      "end": 9742
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/summarization/mod.rs",
        "sha256": "9f9ecfadc332c373eb0fe04b2e9b053e2c6ab205a93b86c6bb7a774a9f5bf2ce"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "variant": "Both",
    "signature": [],
    "returnType": "enum:formal_ai::meta_construction::RecursionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2330,
      "end": 2646
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::Down",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "variant": "Down",
    "returnType": "enum:formal_ai::meta_construction::RecursionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2330,
      "end": 2646
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::Up",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "variant": "Up",
    "returnType": "enum:formal_ai::meta_construction::RecursionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2330,
      "end": 2646
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::Both",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "variant": "Both",
    "returnType": "enum:formal_ai::meta_construction::RecursionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2330,
      "end": 2646
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::emits_downward",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "signature": [
      "enum:formal_ai::meta_construction::RecursionMode"
    ],
    "name": "emits_downward",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Down",
        "value": true
      },
      {
        "variant": "Up",
        "value": false
      },
      {
        "variant": "Both",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2648,
      "end": 3658
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_construction::RecursionMode::emits_upward",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::meta_construction::RecursionMode",
    "signature": [
      "enum:formal_ai::meta_construction::RecursionMode"
    ],
    "name": "emits_upward",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Down",
        "value": false
      },
      {
        "variant": "Up",
        "value": true
      },
      {
        "variant": "Both",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2648,
      "end": 3658
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_construction.rs",
        "sha256": "be7a7e210f846dd660eb21c675b08ebf41b56e5201c50d01be017c3e1ac40023"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_self_improvement::SelfImprovementMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::meta_self_improvement::SelfImprovementMode",
    "variant": "Propose",
    "signature": [],
    "returnType": "enum:formal_ai::meta_self_improvement::SelfImprovementMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2650,
      "end": 2925
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_self_improvement.rs",
        "sha256": "efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_self_improvement::SelfImprovementMode::Off",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::meta_self_improvement::SelfImprovementMode",
    "variant": "Off",
    "returnType": "enum:formal_ai::meta_self_improvement::SelfImprovementMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2650,
      "end": 2925
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_self_improvement.rs",
        "sha256": "efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_self_improvement::SelfImprovementMode::Propose",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::meta_self_improvement::SelfImprovementMode",
    "variant": "Propose",
    "returnType": "enum:formal_ai::meta_self_improvement::SelfImprovementMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2650,
      "end": 2925
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_self_improvement.rs",
        "sha256": "efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::meta_self_improvement::SelfImprovementMode::proposes",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::meta_self_improvement::SelfImprovementMode",
    "signature": [
      "enum:formal_ai::meta_self_improvement::SelfImprovementMode"
    ],
    "name": "proposes",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Off",
        "value": false
      },
      {
        "variant": "Propose",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2927,
      "end": 3670
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/meta_self_improvement.rs",
        "sha256": "efaca24676721c5046257f69677f51dd9105ca914d07e6a963949d4907d7909a"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::selection::SelectionMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::selection::SelectionMode",
    "variant": "Record",
    "signature": [],
    "returnType": "enum:formal_ai::selection::SelectionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 1899,
      "end": 2116
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/selection.rs",
        "sha256": "10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::selection::SelectionMode::Off",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::selection::SelectionMode",
    "variant": "Off",
    "returnType": "enum:formal_ai::selection::SelectionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 1899,
      "end": 2116
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/selection.rs",
        "sha256": "10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::selection::SelectionMode::Record",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::selection::SelectionMode",
    "variant": "Record",
    "returnType": "enum:formal_ai::selection::SelectionMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 1899,
      "end": 2116
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/selection.rs",
        "sha256": "10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::selection::SelectionMode::emits_artifact",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::selection::SelectionMode",
    "signature": [
      "enum:formal_ai::selection::SelectionMode"
    ],
    "name": "emits_artifact",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Off",
        "value": false
      },
      {
        "variant": "Record",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2118,
      "end": 2860
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/selection.rs",
        "sha256": "10dcbf22d0368b08c1e9bd6e3744a7e1b23130422cabf46758d4fed8fa0ce9aa"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::skill_ledger::SkillMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::skill_ledger::SkillMode",
    "variant": "Accumulate",
    "signature": [],
    "returnType": "enum:formal_ai::skill_ledger::SkillMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2439,
      "end": 2678
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/skill_ledger.rs",
        "sha256": "80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::skill_ledger::SkillMode::Off",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::skill_ledger::SkillMode",
    "variant": "Off",
    "returnType": "enum:formal_ai::skill_ledger::SkillMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2439,
      "end": 2678
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/skill_ledger.rs",
        "sha256": "80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::skill_ledger::SkillMode::Accumulate",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::skill_ledger::SkillMode",
    "variant": "Accumulate",
    "returnType": "enum:formal_ai::skill_ledger::SkillMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2439,
      "end": 2678
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/skill_ledger.rs",
        "sha256": "80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::skill_ledger::SkillMode::emits_ledger",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::skill_ledger::SkillMode",
    "signature": [
      "enum:formal_ai::skill_ledger::SkillMode"
    ],
    "name": "emits_ledger",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Off",
        "value": false
      },
      {
        "variant": "Accumulate",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 2680,
      "end": 3426
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/skill_ledger.rs",
        "sha256": "80bdc09278efd0df9a6b9fcea0ee6e36b035644ec420b3411f3be1b50e4c866f"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::world_model_dialog::WorldModelMode::default",
    "kind": "nativeEnumDefault",
    "enumPath": "formal_ai::world_model_dialog::WorldModelMode",
    "variant": "Off",
    "signature": [],
    "returnType": "enum:formal_ai::world_model_dialog::WorldModelMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3768,
      "end": 4017
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/world_model_dialog.rs",
        "sha256": "4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::world_model_dialog::WorldModelMode::Off",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::world_model_dialog::WorldModelMode",
    "variant": "Off",
    "returnType": "enum:formal_ai::world_model_dialog::WorldModelMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3768,
      "end": 4017
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/world_model_dialog.rs",
        "sha256": "4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::world_model_dialog::WorldModelMode::Track",
    "kind": "nativeEnumVariant",
    "enumPath": "formal_ai::world_model_dialog::WorldModelMode",
    "variant": "Track",
    "returnType": "enum:formal_ai::world_model_dialog::WorldModelMode",
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 3768,
      "end": 4017
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/world_model_dialog.rs",
        "sha256": "4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  },
  {
    "path": "formal_ai::world_model_dialog::WorldModelMode::emits_artifact",
    "kind": "nativeEnumScalarMatch",
    "enumPath": "formal_ai::world_model_dialog::WorldModelMode",
    "signature": [
      "enum:formal_ai::world_model_dialog::WorldModelMode"
    ],
    "name": "emits_artifact",
    "nativeType": "bool",
    "returnType": "boolean",
    "cases": [
      {
        "variant": "Off",
        "value": false
      },
      {
        "variant": "Track",
        "value": true
      }
    ],
    "nativeSourceQualified": true,
    "sourceSpan": {
      "start": 4019,
      "end": 4776
    },
    "sourceWitnesses": [
      {
        "path": "rust/src/world_model_dialog.rs",
        "sha256": "4ee687e73e1ee2ed0ada53ff88bc35dae687a974729cf0b76e20d9c011ee7596"
      },
      {
        "path": "rust/src/lib.rs",
        "sha256": "a347856b9542ecb1583fcb9ababf453217afd2267234f380b42092c348f9b224"
      }
    ]
  }
];
