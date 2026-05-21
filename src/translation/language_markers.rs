pub fn detect_source_language(normalized: &str) -> Option<&'static str> {
    if normalized.contains("from english")
        || normalized.contains("с английского")
        || normalized.contains("अंग्रेजी से")
        || normalized.contains("अंग्रेज़ी से")
        || normalized.contains("从英语")
        || normalized.contains("从英文")
    {
        return Some("en");
    }
    if normalized.contains("from russian")
        || normalized.contains("с русского")
        || normalized.contains("रूसी से")
        || normalized.contains("从俄语")
    {
        return Some("ru");
    }
    if normalized.contains("from hindi")
        || normalized.contains("हिंदी से")
        || normalized.contains("हिन्दी से")
        || normalized.contains("从印地语")
        || normalized.contains("从印地文")
    {
        return Some("hi");
    }
    if normalized.contains("from chinese")
        || normalized.contains("चीनी से")
        || normalized.contains("从中文")
        || normalized.contains("从汉语")
        || normalized.contains("从漢語")
    {
        return Some("zh");
    }
    None
}

pub fn detect_target_language(normalized: &str) -> Option<&'static str> {
    if normalized.contains("to english")
        || normalized.contains("на английский")
        || normalized.contains("на английском")
        || normalized.contains("अंग्रेजी में")
        || normalized.contains("अंग्रेज़ी में")
        || [
            "成英文",
            "成英语",
            "为英文",
            "为英语",
            "為英文",
            "為英语",
            "到英文",
            "到英语",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("en");
    }
    if normalized.contains("to russian")
        || normalized.contains("на русский")
        || normalized.contains("रूसी में")
        || [
            "成俄语",
            "成俄語",
            "为俄语",
            "为俄語",
            "為俄语",
            "為俄語",
            "到俄语",
            "到俄語",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("ru");
    }
    if normalized.contains("to hindi")
        || normalized.contains("на хинди")
        || normalized.contains("हिंदी में")
        || normalized.contains("हिन्दी में")
        || [
            "成印地语",
            "成印地文",
            "为印地语",
            "为印地文",
            "為印地语",
            "為印地文",
            "到印地语",
            "到印地文",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("hi");
    }
    if normalized.contains("to chinese")
        || normalized.contains("на китайский")
        || normalized.contains("चीनी में")
        || [
            "成中文",
            "成汉语",
            "成漢語",
            "为中文",
            "为汉语",
            "为漢語",
            "為中文",
            "為汉语",
            "為漢語",
            "到中文",
            "到汉语",
            "到漢語",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("zh");
    }
    None
}
