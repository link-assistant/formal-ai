pub(super) fn filter_csv(content: &str, column: &str, expected: &str) -> Result<String, String> {
    let mut rows = content.lines();
    let header = rows.next().ok_or_else(|| "csv_header_missing".to_owned())?;
    let headers = header.split(',').collect::<Vec<_>>();
    let index = headers
        .iter()
        .position(|name| *name == column)
        .ok_or_else(|| format!("csv_column_missing:{column}"))?;
    let mut selected = vec![header.to_owned()];
    selected.extend(rows.filter_map(|row| {
        let cells = row.split(',').collect::<Vec<_>>();
        (cells.get(index) == Some(&expected)).then(|| row.to_owned())
    }));
    Ok(format!("{}\n", selected.join("\n")))
}

pub(super) fn unique_csv(content: &str, column: &str) -> Result<String, String> {
    let mut rows = content.lines();
    let header = rows.next().ok_or_else(|| "csv_header_missing".to_owned())?;
    let headers = header.split(',').collect::<Vec<_>>();
    let index = headers
        .iter()
        .position(|name| *name == column)
        .ok_or_else(|| format!("csv_column_missing:{column}"))?;
    let mut values = rows
        .filter_map(|row| row.split(',').nth(index).map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Ok(format!("{}\n", values.join("\n")))
}

pub(super) fn query_html(html: &str, selector: &str) -> Result<Vec<String>, String> {
    if let Some(id) = selector.strip_prefix('#') {
        for quote in ['"', '\''] {
            let marker = format!("id={quote}{id}{quote}");
            if let Some(attribute) = html.find(&marker) {
                let open = html[..attribute]
                    .rfind('<')
                    .ok_or_else(|| format!("selector_not_found:{selector}"))?;
                let tag_end = html[open + 1..]
                    .find(|character: char| character.is_whitespace() || character == '>')
                    .map(|index| open + 1 + index)
                    .ok_or_else(|| format!("selector_not_found:{selector}"))?;
                let tag = &html[open + 1..tag_end];
                return extract_tag_contents(&html[open..], tag)
                    .map(|value| vec![strip_tags(value).trim().to_owned()]);
            }
        }
        return Err(format!("selector_not_found:{selector}"));
    }
    if selector.starts_with('.') {
        return Err(format!("selector_unsupported_in_fixture_parser:{selector}"));
    }
    extract_tag_contents(html, selector).map(|value| vec![strip_tags(value).trim().to_owned()])
}

fn extract_tag_contents<'a>(html: &'a str, tag: &str) -> Result<&'a str, String> {
    let open = format!("<{tag}");
    let start = html
        .find(&open)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    let content_start = html[start..]
        .find('>')
        .map(|offset| start + offset + 1)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    let close = format!("</{tag}>");
    let content_end = html[content_start..]
        .find(&close)
        .map(|offset| content_start + offset)
        .ok_or_else(|| format!("selector_not_found:{tag}"))?;
    Ok(&html[content_start..content_end])
}

fn strip_tags(value: &str) -> String {
    let mut output = String::new();
    let mut inside = false;
    for character in value.chars() {
        match character {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => output.push(character),
            _ => {}
        }
    }
    output
}
