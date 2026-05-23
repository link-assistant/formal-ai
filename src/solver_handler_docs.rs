//! Scoped documentation-answer handlers for known project APIs.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::solver_handlers::finalize_simple;

const PANDAS_DATAFRAME_JOIN_DOCS_URL: &str =
    "https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html";

/// Handles project-method documentation prompts such as
/// "how the join method works in pandas" with a narrow official-docs summary.
pub fn try_docs_method_explanation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !is_pandas_dataframe_join_prompt(prompt, normalized) {
        return None;
    }

    log.append("docs_method:request", prompt.to_owned());
    log.append("docs_method:project", "pandas".to_owned());
    log.append("docs_method:method", "pandas.DataFrame.join".to_owned());
    log.append("docs_method:source_kind", "official-docs".to_owned());
    log.append("docs_method:source", PANDAS_DATAFRAME_JOIN_DOCS_URL);

    let body = render_pandas_dataframe_join_answer(detect_language(prompt));

    Some(finalize_simple(
        prompt,
        log,
        "docs_method_explanation",
        "response:docs_method_explanation",
        &body,
        0.92,
    ))
}

fn is_pandas_dataframe_join_prompt(prompt: &str, normalized: &str) -> bool {
    let lower = prompt.to_lowercase();
    let normalized = normalized.trim();
    if is_explicit_web_search(normalized) {
        return false;
    }
    if !has_word(normalized, "pandas") {
        return false;
    }
    if !is_explanation_request(normalized) {
        return false;
    }

    lower.contains("dataframe.join")
        || lower.contains("df.join")
        || normalized.contains("join method")
        || normalized.contains("method join")
        || (has_word(normalized, "join") && has_word(normalized, "метод"))
        || (has_word(normalized, "join") && normalized.contains("विधि"))
        || (has_word(normalized, "join") && normalized.contains("方法"))
        || (has_word(normalized, "join")
            && (has_word(normalized, "method") || has_word(normalized, "dataframe")))
}

fn is_explicit_web_search(normalized: &str) -> bool {
    let requests_search = normalized.starts_with("search ")
        || normalized.starts_with("find ")
        || normalized.starts_with("look up ")
        || normalized.starts_with("lookup ");
    requests_search
        && (has_word(normalized, "web")
            || has_word(normalized, "internet")
            || has_word(normalized, "online"))
}

fn is_explanation_request(normalized: &str) -> bool {
    normalized.starts_with("how ")
        || normalized.contains(" how ")
        || normalized.starts_with("explain ")
        || normalized.starts_with("describe ")
        || normalized.starts_with("what does ")
        || normalized.starts_with("what is ")
        || normalized.starts_with("tell me about ")
        || normalized.starts_with("how to use ")
        || normalized.starts_with("как ")
        || normalized.contains(" как ")
        || normalized.starts_with("объясни ")
        || normalized.starts_with("расскажи ")
        || normalized.starts_with("что такое ")
        || normalized.contains("कैसे काम")
        || normalized.starts_with("समझाओ")
        || normalized.starts_with("क्या है ")
        || normalized.contains("如何工作")
        || normalized.contains("怎么工作")
        || normalized.starts_with("解释")
        || normalized.contains("是什么")
}

fn has_word(normalized: &str, word: &str) -> bool {
    normalized.split_whitespace().any(|token| token == word)
}

fn render_pandas_dataframe_join_answer(language: Language) -> String {
    match language {
        Language::Russian => format!(
            "pandas `DataFrame.join` добавляет столбцы из `other` DataFrame \
             или именованной Series к вызывающему DataFrame и возвращает новый \
             DataFrame.\n\n\
             В рамках этого метода: по умолчанию это left join по индексу \
             вызывающего DataFrame. Если задан `on`, pandas сопоставляет этот \
             столбец или уровень индекса с индексом объекта `other`. Параметр \
             `how` управляет объединением ключей (`left`, `right`, `outer`, \
             `inner`, `cross`, `left_anti` или `right_anti`). `lsuffix` и \
             `rsuffix` нужны при совпадающих именах столбцов, `sort` сортирует \
             ключи join, а `validate` проверяет связи one-to-one, one-to-many, \
             many-to-one или many-to-many. Для join столбец-к-столбцу \
             документация pandas указывает на `DataFrame.merge`.\n\n\
             Источник: [pandas.DataFrame.join]({PANDAS_DATAFRAME_JOIN_DOCS_URL}) \
             (официальная документация pandas)."
        ),
        Language::Hindi => format!(
            "pandas `DataFrame.join` कॉल करने वाले DataFrame में `other` \
             DataFrame या named Series के columns जोड़ता है और नया DataFrame \
             लौटाता है.\n\n\
             इस method के दायरे में: default रूप से यह caller के index पर left \
             join करता है. `on` देने पर pandas caller के उस column या index \
             level को `other` object के index से मिलाता है. `how` parameter \
             keys को मिलाने का तरीका चुनता है (`left`, `right`, `outer`, \
             `inner`, `cross`, `left_anti`, या `right_anti`). Column नाम टकराने \
             पर `lsuffix` और `rsuffix`, join keys को sort करने के लिए `sort`, \
             और one-to-one, one-to-many, many-to-one, या many-to-many संबंध \
             जांचने के लिए `validate` इस्तेमाल करें. Column-on-column joins \
             के लिए pandas docs `DataFrame.merge` की ओर भेजते हैं.\n\n\
             Source: [pandas.DataFrame.join]({PANDAS_DATAFRAME_JOIN_DOCS_URL}) \
             (official pandas docs)."
        ),
        Language::Chinese => format!(
            "pandas `DataFrame.join` 会把 `other` DataFrame 或具名 Series 的列加入调用方，并返回新的 DataFrame。\n\n\
             只看这个方法：默认情况下，它使用调用方的 index 执行 left join。设置 `on` 时，pandas 会把调用方的列或索引层级与 `other` 对象的 index 匹配。`how` 参数控制键的组合方式（`left`、`right`、`outer`、`inner`、`cross`、`left_anti` 或 `right_anti`）。列名冲突时使用 `lsuffix` 和 `rsuffix`，用 `sort` 排序 join keys，用 `validate` 检查 one-to-one、one-to-many、many-to-one 或 many-to-many 关系。对于列到列的 join，pandas 文档指向 `DataFrame.merge`。\n\n\
             Source: [pandas.DataFrame.join]({PANDAS_DATAFRAME_JOIN_DOCS_URL}) \
             (official pandas docs)."
        ),
        Language::English | Language::Unknown => format!(
            "pandas `DataFrame.join` joins columns from the `other` DataFrame or \
             named Series into the caller and returns a new DataFrame.\n\n\
             Scoped to this method: by default, it performs a left join using the \
             caller's index. If `on` is set, pandas matches that caller column or \
             index level against the `other` object's index. The `how` parameter \
             controls key handling (`left`, `right`, `outer`, `inner`, `cross`, \
             `left_anti`, or `right_anti`). Use `lsuffix` and `rsuffix` when \
             column names overlap, `sort` to order join keys, and `validate` to \
             check one-to-one, one-to-many, many-to-one, or many-to-many \
             relationships. For column-on-column joins, the pandas docs point to \
             `DataFrame.merge`.\n\n\
             Source: [pandas.DataFrame.join]({PANDAS_DATAFRAME_JOIN_DOCS_URL}) \
             (official pandas docs)."
        ),
    }
}
