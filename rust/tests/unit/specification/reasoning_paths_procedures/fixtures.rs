//! Source-backed procedure fixtures; assertion bodies stay in their original tests.

pub(super) fn unverified_procedure(language: &str, task: &str) -> String {
    match language {
        "en" => format!(
            "I couldn’t verify steps for “{task}”. If you have a guide, share its link and I can help you work through it."
        ),
        "ru" => format!(
            "Не удалось найти проверенные инструкции по запросу «{task}». Если у вас есть ссылка на руководство, пришлите её."
        ),
        "hi" => format!(
            "मुझे “{task}” के लिए सत्यापित निर्देश नहीं मिले। अगर आपके पास किसी मार्गदर्शिका का लिंक है, तो उसे भेजें।"
        ),
        "zh" => format!("我没能核实「{task}」的操作步骤。如果你有相关指南，请发给我链接。"),
        other => panic!("missing unverified procedure language {other}"),
    }
}

pub(super) struct InstallCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
    pub(super) task_fragment: &'static str,
    pub(super) object_fragment: &'static str,
}

pub(super) const INSTALL_CASES: [InstallCase; 4] = [
    InstallCase {
        language: "en",
        prompt: "how install cursor",
        task_fragment: "install cursor",
        object_fragment: "cursor",
    },
    InstallCase {
        language: "ru",
        prompt: "как установить cursor",
        task_fragment: "установить cursor",
        object_fragment: "cursor",
    },
    InstallCase {
        language: "hi",
        prompt: "कैसे इंस्टॉल करें cursor",
        task_fragment: "इंस्टॉल करें cursor",
        object_fragment: "cursor",
    },
    InstallCase {
        language: "zh",
        prompt: "如何安装 cursor",
        task_fragment: "安装 cursor",
        object_fragment: "cursor",
    },
];

pub(super) struct SurfaceCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
}

pub(super) const SURFACE_CASES: [SurfaceCase; 4] = [
    SurfaceCase {
        language: "en",
        prompt: "how to do 3d print ordering",
    },
    SurfaceCase {
        language: "ru",
        prompt: "как сделать 3d print ordering",
    },
    SurfaceCase {
        language: "hi",
        prompt: "कैसे करें 3d print ordering",
    },
    SurfaceCase {
        language: "zh",
        prompt: "如何做 3d print ordering",
    },
];

pub(super) struct TypoCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
}

pub(super) const TYPO_CASES: [TypoCase; 4] = [
    TypoCase {
        language: "en",
        prompt: "How to do SPEC dirven development step by step?",
    },
    TypoCase {
        language: "ru",
        prompt: "как сделать SPEC dirven development? напиши по шагам",
    },
    TypoCase {
        language: "hi",
        prompt: "कैसे करें SPEC dirven development? चरणों में बताओ",
    },
    TypoCase {
        language: "zh",
        prompt: "如何做 SPEC dirven development？按步骤写",
    },
];
