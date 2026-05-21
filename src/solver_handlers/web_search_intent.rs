//! Natural-language web-search intent recognition.

use crate::engine::normalize_prompt;

use super::web_requests::normalize_url_candidate;

pub(super) fn extract_web_search_query(prompt: &str, normalized: &str) -> Option<String> {
    let normalized_words = normalize_prompt(prompt);
    if normalized_words.starts_with("search conversations ")
        || normalized_words.starts_with("search my conversations ")
        || normalized_words.starts_with("search my chats ")
    {
        return None;
    }
    for prefix in WEB_SEARCH_EXPLICIT_PREFIXES {
        if let Some(query) = normalized_words.strip_prefix(prefix) {
            if let Some(query) = valid_search_query(query) {
                return Some(query);
            }
        }
        if let Some(query) = normalized.strip_prefix(prefix) {
            if let Some(query) = valid_search_query(query) {
                return Some(query);
            }
        }
    }
    extract_semantic_web_search_query(&normalized_words)
}

fn clean_search_query(value: &str) -> String {
    value
        .trim()
        .trim_matches(is_url_wrapper_punctuation)
        .trim_end_matches(is_url_trailing_punctuation)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const fn is_url_wrapper_punctuation(character: char) -> bool {
    matches!(
        character,
        '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`' | '«' | '»'
    )
}

const fn is_url_trailing_punctuation(character: char) -> bool {
    matches!(character, '.' | ',' | '!' | '?' | ';' | ':' | '…')
}

const WEB_SEARCH_EXPLICIT_PREFIXES: &[&str] = &[
    "search the web for ",
    "search web for ",
    "search the internet for ",
    "search internet for ",
    "search online for ",
    "search for information about ",
    "search for information on ",
    "web search for ",
    "find on the internet ",
    "find online ",
    "find information about ",
    "find information on ",
    "find detailed information about ",
    "find detailed information on ",
    "find info about ",
    "find info on ",
    "look up information about ",
    "look up information on ",
    "look up info about ",
    "look up info on ",
    "look up online ",
    "найди в интернете ",
    "поищи в интернете ",
    "поиск в интернете ",
    "найди онлайн ",
    "поищи онлайн ",
    "найди в сети ",
    "поищи в сети ",
    "найди информацию в интернете о ",
    "найди информацию в интернете об ",
    "поищи информацию в интернете о ",
    "поищи информацию в интернете об ",
    "найди информацию о ",
    "найди информацию об ",
    "найди информацию про ",
    "найди информацию по ",
    "найти информацию о ",
    "найти информацию об ",
    "поищи информацию о ",
    "поищи информацию об ",
    "поищи информацию про ",
    "поищи информацию по ",
    "найди инфу о ",
    "найди инфу об ",
    "поищи инфу о ",
    "поищи инфу об ",
    "найди сведения о ",
    "найди сведения об ",
    "поищи сведения о ",
    "поищи сведения об ",
    "найди материалы о ",
    "найди материалы об ",
    "поищи материалы о ",
    "поищи материалы об ",
];

const WEB_SEARCH_ACTION_MARKERS: &[&str] = &[
    " search ",
    " find ",
    " look up ",
    " lookup ",
    " research ",
    " investigate ",
    " найди ",
    " найти ",
    " поищи ",
    " поиск ",
    " поискать ",
    " ищи ",
    " разыщи ",
    " узнай ",
    "खोज",
    "ढूंढ",
    "ढूँढ",
    "搜索",
    "查找",
    "查询",
    "檢索",
    "检索",
    "搜一下",
    "查一下",
];

const WEB_SEARCH_STRONG_ACTION_MARKERS: &[&str] = &[
    " search ",
    " look up ",
    " lookup ",
    " research ",
    " investigate ",
    " поищи ",
    " поиск ",
    " поискать ",
    " ищи ",
    "खोज",
    "ढूंढ",
    "ढूँढ",
    "搜索",
    "查找",
    "查询",
    "檢索",
    "检索",
    "搜一下",
    "查一下",
];

const WEB_SEARCH_SIGNAL_MARKERS: &[&str] = &[
    " web ",
    " internet ",
    " online ",
    " wikipedia ",
    " wikidata ",
    " wiktionary ",
    " information ",
    " info ",
    " details ",
    " data ",
    " material ",
    " materials ",
    " resource ",
    " resources ",
    " source ",
    " sources ",
    " article ",
    " articles ",
    " fact ",
    " facts ",
    " интернете ",
    " интернет ",
    " онлайн ",
    " сети ",
    " википед",
    " викиданн",
    " информац",
    " инфу ",
    " сведения ",
    " материал",
    " данные ",
    " источник",
    "जानकारी",
    "सूचना",
    "विवरण",
    "सामग्री",
    "स्रोत",
    "लेख",
    "इंटरनेट",
    "ऑनलाइन",
    "वेब",
    "विकिपीडिया",
    "विकिडाटा",
    "信息",
    "資料",
    "资料",
    "内容",
    "來源",
    "来源",
    "资源",
    "資源",
    "文章",
    "百科",
    "维基百科",
    "維基百科",
    "维基数据",
    "維基數據",
    "网上",
    "網上",
    "在线",
    "在線",
    "互联网",
    "網路",
    "网络",
];

const SEARCH_QUERY_AFTER_MARKERS: &[&str] = &[
    " about ",
    " on ",
    " regarding ",
    " concerning ",
    " for ",
    " о ",
    " об ",
    " про ",
    " по ",
    " насчет ",
    " относительно ",
    "关于",
    "關於",
    "有关",
    "有關",
];

const SEARCH_QUERY_BEFORE_MARKERS: &[&str] = &[
    " के बारे में",
    " के विषय में",
    " से संबंधित",
    " पर",
    " की जानकारी",
    " की सूचना",
];

const SEARCH_ACTION_AFTER_MARKERS: &[&str] = &[
    "search for ",
    "search ",
    "find ",
    "look up ",
    "lookup ",
    "research ",
    "investigate ",
    "найди ",
    "найти ",
    "поищи ",
    "поискать ",
    "ищи ",
    "разыщи ",
    "узнай ",
    "खोजो ",
    "खोजें ",
    "खोजिए ",
    "ढूंढो ",
    "ढूँढो ",
    "ढूंढें ",
    "ढूँढें ",
    "搜索",
    "查找",
    "查询",
    "檢索",
    "检索",
    "搜一下",
    "查一下",
];

const SEARCH_QUERY_LEADING_NOISE: &[&str] = &[
    "please ",
    "can you ",
    "could you ",
    "would you ",
    "me ",
    "the ",
    "some ",
    "detailed ",
    "more ",
    "current ",
    "latest ",
    "information about ",
    "information on ",
    "info about ",
    "info on ",
    "details about ",
    "details on ",
    "data about ",
    "data on ",
    "подробные ",
    "информацию о ",
    "информацию об ",
    "инфу о ",
    "инфу об ",
    "сведения о ",
    "сведения об ",
    "материалы о ",
    "материалы об ",
    "материалы по ",
    "данные о ",
    "данные об ",
    "о ",
    "об ",
    "про ",
    "по ",
    "कृपया ",
    "जानकारी ",
    "सूचना ",
    "विवरण ",
    "सामग्री ",
    "关于",
    "關於",
    "有关",
    "有關",
];

const SEARCH_QUERY_TRAILING_NOISE: &[&str] = &[
    " online",
    " on the internet",
    " on the web",
    " on wikipedia",
    " in wikipedia",
    " from wikipedia",
    " information",
    " info",
    " details",
    " data",
    " material",
    " materials",
    " resources",
    " sources",
    " articles",
    " facts",
    " в интернете",
    " онлайн",
    " в сети",
    " в википедии",
    " википедии",
    " информация",
    " сведения",
    " материалы",
    " данные",
    " के बारे में",
    " के विषय में",
    " से संबंधित",
    " पर",
    " की जानकारी",
    " की सूचना",
    " जानकारी",
    " सूचना",
    " विवरण",
    " सामग्री",
    " स्रोत",
    " विकिपीडिया में",
    " ऑनलाइन",
    " इंटरनेट पर",
    " खोजो",
    " खोजें",
    " खोजिए",
    " ढूंढो",
    " ढूँढो",
    " ढूंढें",
    " ढूँढें",
    "的信息",
    "的資料",
    "的资料",
    "信息",
    "資料",
    "资料",
    "内容",
    "文章",
    "在维基百科上",
    "在維基百科上",
    "维基百科",
    "維基百科",
    "网上",
    "網上",
    "在线",
    "在線",
    "搜索",
    "查找",
    "查一下",
    "搜一下",
];

const SEARCH_QUERY_SOURCE_ONLY: &[&str] = &[
    "web",
    "internet",
    "online",
    "wikipedia",
    "wikidata",
    "wiktionary",
    "интернет",
    "интернете",
    "онлайн",
    "сети",
    "википедии",
    "इंटरनेट",
    "ऑनलाइन",
    "वेब",
    "विकिपीडिया",
    "网上",
    "網上",
    "在线",
    "在線",
    "互联网",
    "網路",
    "网络",
    "维基百科",
    "維基百科",
];

fn extract_semantic_web_search_query(normalized: &str) -> Option<String> {
    let has_action = contains_any_search_marker(normalized, WEB_SEARCH_ACTION_MARKERS);
    if !has_action {
        return None;
    }
    let has_strong_action =
        contains_any_search_marker(normalized, WEB_SEARCH_STRONG_ACTION_MARKERS);
    if !has_strong_action && !contains_any_search_marker(normalized, WEB_SEARCH_SIGNAL_MARKERS) {
        return None;
    }
    for marker in SEARCH_QUERY_AFTER_MARKERS {
        if let Some(index) = normalized.find(marker) {
            let start = index + marker.len();
            if let Some(query) = valid_search_query(&normalized[start..]) {
                return Some(query);
            }
        }
    }
    for marker in SEARCH_QUERY_BEFORE_MARKERS {
        if let Some(index) = normalized.find(marker) {
            if let Some(query) = valid_search_query(&normalized[..index]) {
                return Some(query);
            }
        }
    }
    for marker in SEARCH_ACTION_AFTER_MARKERS {
        if let Some(index) = normalized.find(marker) {
            let start = index + marker.len();
            if let Some(query) = valid_search_query(&normalized[start..]) {
                return Some(query);
            }
        }
    }
    None
}

fn contains_any_search_marker(normalized: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .any(|marker| contains_search_marker(normalized, marker))
}

fn contains_search_marker(normalized: &str, marker: &str) -> bool {
    if marker.starts_with(' ') || marker.ends_with(' ') {
        let padded = format!(" {normalized} ");
        padded.contains(marker)
    } else {
        normalized.contains(marker)
    }
}

fn valid_search_query(value: &str) -> Option<String> {
    let query = clean_semantic_search_query(value);
    let query_key = query.to_lowercase();
    if query.is_empty()
        || SEARCH_QUERY_SOURCE_ONLY.contains(&query_key.as_str())
        || normalize_url_candidate(&query).is_some()
    {
        return None;
    }
    Some(query)
}

fn clean_semantic_search_query(value: &str) -> String {
    let mut query = clean_search_query(value);
    loop {
        let before = query.clone();
        for prefix in SEARCH_QUERY_LEADING_NOISE {
            if let Some(stripped) = query.strip_prefix(prefix) {
                query = clean_search_query(stripped);
            }
        }
        for suffix in SEARCH_QUERY_TRAILING_NOISE {
            if let Some(stripped) = query.strip_suffix(suffix) {
                query = clean_search_query(stripped);
            }
        }
        if query == before {
            return query;
        }
    }
}
