//! Scoped documentation-answer handlers for known project APIs.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::seed::{self, Slot};
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

/// True when the prompt asks how the pandas `DataFrame.join` method works.
///
/// The prompt must address pandas, read as an [`is_explanation_request`], and not
/// be an [`is_explicit_web_search`]. The join itself is then recognised through
/// two kinds of evidence:
///
/// * Code-resident API identifiers — `DataFrame.join`, `df.join`, and the
///   `join`+`dataframe` pairing. These are written the same in every language
///   (they are the library's own symbol names, not natural-language words), so
///   they legitimately live in the handler rather than the lexicon: they are the
///   bridge from a multilingual question to one specific documented API.
/// * The translatable noun "method" — matched through the
///   [`code_method_noun`](seed::ROLE_CODE_METHOD_NOUN) role rather than the four
///   per-language words it used to hardcode, paired with the `join` identifier so
///   "how the join method works", "метод join", "join विधि", "join 方法" all
///   resolve without naming the word "method" in any language here.
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
        || (has_word(normalized, "join") && has_word(normalized, "dataframe"))
        || (has_word(normalized, "join")
            && seed::lexicon().mentions_role(seed::ROLE_CODE_METHOD_NOUN, normalized))
}

/// True when the prompt opens with an imperative to search the web, so it should
/// be answered by the web-search handler rather than this narrow docs handler.
///
/// Mirrors the original two-part screen by meaning. The search imperative is read
/// from the [`web_search_imperative_lead`](seed::ROLE_WEB_SEARCH_IMPERATIVE_LEAD)
/// role: only its [`Slot::Prefix`] forms are clause-initial leads, so the literal
/// before each ellipsis ("search ", "look up ", "research ", "найди ", "搜索", …)
/// is matched against the start of the prompt — exactly the original `starts_with`
/// test, now covering every supported language. The medium is read from the
/// [`web_medium`](seed::ROLE_WEB_MEDIUM) role; its surfaces are space-wrapped, so
/// they are matched with the web-search recogniser's whole-token convention — pad
/// the prompt with spaces and test `contains` — which also catches a medium word
/// at the very end ("search the web").
fn is_explicit_web_search(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    let requests_search = lexicon
        .role_word_forms(seed::ROLE_WEB_SEARCH_IMPERATIVE_LEAD)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .any(|form| normalized.starts_with(form.before_slot()));
    if !requests_search {
        return false;
    }
    let padded = format!(" {normalized} ");
    lexicon
        .role_word_forms(seed::ROLE_WEB_MEDIUM)
        .into_iter()
        .any(|form| {
            let marker = form.text.as_str();
            if marker.starts_with(' ') || marker.ends_with(' ') {
                padded.contains(marker)
            } else {
                normalized.contains(marker)
            }
        })
}

/// True when the prompt is phrased as a request to have something explained.
///
/// Every interrogative and imperative lead-in lives in the
/// [`explanation_request_lead`](seed::ROLE_EXPLANATION_REQUEST_LEAD) role rather
/// than in this function, so no question word is hardcoded here. Each surface is
/// matched by its slot, mirroring `meta_explanation`'s `is_why_question`: a
/// [`Slot::Prefix`] form ("how …", "explain …", "как …", "क्या है …", "解释…") is
/// matched by the literal before the ellipsis against the start of the prompt,
/// while a bare form ("how", "कैसे काम", "如何工作", …) is matched as a raw
/// substring anywhere. The space-wrapped bare forms (" how ", " как ") match only
/// on whole-word boundaries.
fn is_explanation_request(normalized: &str) -> bool {
    seed::lexicon()
        .role_word_forms(seed::ROLE_EXPLANATION_REQUEST_LEAD)
        .into_iter()
        .any(|form| match form.slot() {
            Slot::Prefix => normalized.starts_with(form.before_slot()),
            _ => normalized.contains(form.text.as_str()),
        })
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
