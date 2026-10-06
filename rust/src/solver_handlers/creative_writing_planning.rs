// ---------------------------------------------------------------------------
// Planning
// ---------------------------------------------------------------------------

/// One `planning_place_cache` row: a place with the facts it cites.
#[derive(Clone)]
struct Place {
    id: String,
    name: String,
    district: String,
    category: String,
    hours: String,
    #[allow(dead_code)] // seed provenance, carried but not yet rendered
    closed_days: String,
    visit_minutes: u32,
    source_url: String,
}

/// One `destination_surface` record: the surfaces naming a destination.
fn destinations() -> Vec<(String, Vec<String>)> {
    rules_records_named("destination_surface")
        .map(|record| {
            (
                record.find_child_value("destination").to_string(),
                record
                    .children
                    .iter()
                    .filter(|child| child.name == "surface")
                    .map(|child| child.id.clone())
                    .filter(|surface| !surface.is_empty())
                    .collect(),
            )
        })
        .filter(|(destination, _)| !destination.is_empty())
        .collect()
}

fn places(destination: &str) -> Vec<Place> {
    rules_records_named("planning_place_cache")
        .filter(|record| record.find_child_value("destination") == destination)
        .map(|record| Place {
            id: record.find_child_value("id").to_string(),
            name: record.find_child_value("name").to_string(),
            district: record.find_child_value("district").to_string(),
            category: record.find_child_value("category").to_string(),
            hours: record.find_child_value("hours").to_string(),
            closed_days: record.find_child_value("closed_days").to_string(),
            visit_minutes: record
                .find_child_value("visit_minutes")
                .parse()
                .unwrap_or(60),
            source_url: record.find_child_value("source_url").to_string(),
        })
        .filter(|place| !place.id.is_empty() && !place.source_url.is_empty())
        .collect()
}

/// The `plan_constraint` record: windows, travel times, caps.
struct PlanConstraints {
    travel_intra: u32,
    travel_cross: u32,
    lunch_start: u32,
    lunch_end: u32,
    day_start: u32,
    day_end: u32,
    max_items: usize,
    note: String,
}

fn plan_constraints() -> PlanConstraints {
    let default = PlanConstraints {
        travel_intra: 20,
        travel_cross: 40,
        lunch_start: 13 * 60,
        lunch_end: 14 * 60,
        day_start: 9 * 60,
        day_end: 19 * 60,
        max_items: 4,
        note: String::new(),
    };
    let Some(record) = rules_records_named("plan_constraint").next() else {
        return default;
    };
    let (day_start, day_end) = clock_window(
        record.find_child_value("day_window"),
        default.day_start,
        default.day_end,
    );
    let (lunch_start, lunch_end) = clock_window(
        record.find_child_value("lunch_window"),
        default.lunch_start,
        default.lunch_end,
    );
    PlanConstraints {
        travel_intra: record
            .find_child_value("travel_intra_district_minutes")
            .parse()
            .unwrap_or(default.travel_intra),
        travel_cross: record
            .find_child_value("travel_cross_district_minutes")
            .parse()
            .unwrap_or(default.travel_cross),
        lunch_start,
        lunch_end,
        day_start,
        day_end,
        max_items: record
            .find_child_value("max_items_per_day")
            .parse()
            .unwrap_or(default.max_items),
        note: record.find_child_value("note").to_string(),
    }
}

/// Parse "HH:MM" into minutes past midnight.
fn parse_hhmm(text: &str) -> Option<u32> {
    let (hours, minutes) = text.split_once(':')?;
    Some(hours.trim().parse::<u32>().ok()? * 60 + minutes.trim().parse::<u32>().ok()?)
}

/// The opening and closing minutes of an "HH:MM-HH:MM" window, with the
/// fallbacks standing in for an unparsable end ("always open").
fn clock_window(hours: &str, open: u32, close: u32) -> (u32, u32) {
    (
        hours.split('-').next().and_then(parse_hhmm).unwrap_or(open),
        hours
            .split('-')
            .next_back()
            .and_then(parse_hhmm)
            .unwrap_or(close),
    )
}

/// Render minutes past midnight as "HH:MM".
fn fmt_minutes(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// One scheduled item: the place, its window, and the citation it carries.
struct ScheduledItem {
    start: u32,
    end: u32,
    place: Place,
}

/// Schedule `places` across `days` days under the plan constraints.
///
/// Days take whole districts in seed order (ancient, vatican, centro,
/// trastevere round-robin), items sort early-closing first, and the
/// simulator enforces the windows: an item starts after travel from the
/// previous item (and after the lunch window once it crosses into it),
/// ends before closing, respects the daily cap, and never follows an item
/// of its own category consecutively. Skipped items and their reasons are
/// returned alongside, so the caller can log every scheduling decision.
fn schedule_days(
    available: Vec<Place>,
    days: usize,
    limits: &PlanConstraints,
) -> (Vec<Vec<ScheduledItem>>, Vec<String>) {
    let mut skips: Vec<String> = Vec::new();
    // Group the places by district, preserving seed order.
    let mut district_order: Vec<String> = Vec::new();
    for place in &available {
        if !district_order.contains(&place.district) {
            district_order.push(place.district.clone());
        }
    }
    let mut day_plans: Vec<Vec<Place>> = vec![Vec::new(); days.max(1)];
    for (index, district) in district_order.iter().enumerate() {
        let mut bucket: Vec<&Place> = available
            .iter()
            .filter(|place| place.district == *district)
            .collect();
        // Early-closing places first (markets), long visits before short.
        bucket.sort_by_key(|place| {
            let close = place
                .hours
                .split('-')
                .next_back()
                .and_then(parse_hhmm)
                .unwrap_or(limits.day_end);
            (close, u32::MAX - place.visit_minutes)
        });
        for (slot, place) in bucket.iter().enumerate() {
            let day = (index + slot) % day_plans.len();
            day_plans[day].push((*place).clone());
        }
    }

    let mut out = Vec::new();
    for mut day_places in day_plans {
        if day_places.len() > limits.max_items {
            for place in day_places.drain(limits.max_items..) {
                skips.push(format!("{} (over the daily cap)", place.name));
            }
        }
        let mut items: Vec<ScheduledItem> = Vec::new();
        let mut cursor = limits.day_start;
        let mut previous_district: Option<String> = None;
        let mut lunch_taken = false;
        for place in day_places {
            if let Some(previous) = &previous_district {
                // The category-spread constraint: no two consecutive items
                // of one category on a day.
                if items
                    .last()
                    .is_some_and(|item| item.place.category == place.category)
                {
                    skips.push(format!(
                        "{} (would repeat the {} category consecutively)",
                        place.name, place.category
                    ));
                    continue;
                }
                let travel = if previous == &place.district {
                    limits.travel_intra
                } else {
                    limits.travel_cross
                };
                cursor += travel;
            }
            let open = place
                .hours
                .split('-')
                .next()
                .and_then(parse_hhmm)
                .unwrap_or(limits.day_start);
            let close = place
                .hours
                .split('-')
                .next_back()
                .and_then(parse_hhmm)
                .unwrap_or(limits.day_end);
            let mut start = cursor.max(open);
            if !lunch_taken && start + 30 > limits.lunch_start {
                // The lunch window: one hour, once a day.
                start = start.max(limits.lunch_end);
                lunch_taken = true;
            }
            let end = start + place.visit_minutes;
            if end > close.min(limits.day_end) {
                skips.push(format!(
                    "{} (would run past its {} closing)",
                    place.name, place.hours
                ));
                continue;
            }
            items.push(ScheduledItem {
                start,
                end,
                place: place.clone(),
            });
            cursor = end;
            previous_district = Some(place.district);
        }
        out.push(items);
    }
    (out, skips)
}

/// Verify a finished schedule: no overlaps, travel time between
/// consecutive items, every item inside its hours, and the daily cap.
/// Returns the failures (empty when feasible).
fn check_schedule(schedule: &[Vec<ScheduledItem>], limits: &PlanConstraints) -> Vec<String> {
    let mut failures = Vec::new();
    for (day_index, items) in schedule.iter().enumerate() {
        if items.len() > limits.max_items {
            failures.push(format!(
                "day {} carries {} items, over the cap of {}",
                day_index + 1,
                items.len(),
                limits.max_items
            ));
        }
        for pair in items.windows(2) {
            let (first, second) = (&pair[0], &pair[1]);
            let travel = if first.place.district == second.place.district {
                limits.travel_intra
            } else {
                limits.travel_cross
            };
            if second.start < first.end + travel {
                failures.push(format!(
                    "day {}: {} starts before {} ends plus travel",
                    day_index + 1,
                    second.place.name,
                    first.place.name
                ));
            }
            if first.place.category == second.place.category {
                failures.push(format!(
                    "day {}: two consecutive {} items",
                    day_index + 1,
                    first.place.category
                ));
            }
        }
        for item in items {
            let open = item
                .place
                .hours
                .split('-')
                .next()
                .and_then(parse_hhmm)
                .unwrap_or(limits.day_start);
            let close = item
                .place
                .hours
                .split('-')
                .next_back()
                .and_then(parse_hhmm)
                .unwrap_or(limits.day_end);
            if item.start < open || item.end > close {
                failures.push(format!(
                    "{} falls outside {}",
                    item.place.name, item.place.hours
                ));
            }
        }
    }
    failures
}

/// Recognize a planning request and answer with a cited, feasibility-
/// checked schedule. Returns `None` when the prompt is not a planning
/// request; a destination without seeded places gets the honest refusal,
/// never the canned web-search paragraph or a terminal command.
pub fn handle_planning_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !crate::seed::lexicon().mentions_role(ROLE_PLANNING, normalized) {
        return None;
    }
    let language = crate::language::detect(prompt).slug();
    let destination = destinations()
        .into_iter()
        .find(|(_, surfaces)| {
            surfaces
                .iter()
                .any(|surface| normalized.contains(&surface.to_lowercase()))
        })
        .map(|(destination, _)| destination);
    let Some(destination) = destination else {
        let fallback = topic_words(normalized, &language)
            .into_iter()
            .next()
            .unwrap_or_default();
        let cached = destinations()
            .into_iter()
            .map(|(destination, _)| destination)
            .collect::<Vec<_>>()
            .join(", ");
        log.append("planning:refusal", "destination not cached".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "planning",
            "response:planning",
            &template(
                "planning_refusal",
                &[
                    ("destination", &fallback),
                    ("cached_destinations", &cached),
                ],
            ),
            0.4,
        ));
    };
    log.append("planning:destination", destination.clone());
    let days = requested_count(normalized, &language)
        .unwrap_or(3)
        .clamp(1, 7) as usize;
    log.append("planning:days", days.to_string());
    let limits = plan_constraints();
    let available = places(&destination);
    for place in &available {
        log.append(
            "planning:place",
            format!("{} ({} min, {})", place.name, place.visit_minutes, place.hours),
        );
    }
    let (schedule, skips) = schedule_days(available, days, &limits);
    for skip in &skips {
        log.append("planning:skipped", skip.clone());
    }
    let failures = check_schedule(&schedule, &limits);
    if !failures.is_empty() {
        log.append("planning:infeasible", failures.join("; "));
        return Some(finalize_simple(
            prompt,
            log,
            "planning",
            "response:planning",
            &template(
                "planning_infeasible",
                &[
                    ("days", &days.to_string()),
                    ("destination", &destination),
                    ("failures", &failures.join("; ")),
                ],
            ),
            0.4,
        ));
    }
    let rendered = schedule
        .iter()
        .enumerate()
        .map(|(index, items)| {
            let lines: Vec<String> = items
                .iter()
                .map(|item| {
                    format!(
                        "  {}-{} {} ({}, {}) — {}",
                        fmt_minutes(item.start),
                        fmt_minutes(item.end),
                        item.place.name,
                        item.place.category,
                        item.place.district,
                        item.place.source_url
                    )
                })
                .collect();
            format!("Day {}\n{}", index + 1, lines.join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let feasibility = format!(
        "no overlapping items; travel time accounted between consecutive items ({} min intra-district, {} min cross-district); every item inside its opening hours; at most {} items a day",
        limits.travel_intra, limits.travel_cross, limits.max_items
    );
    let body = template(
        "planning_itinerary",
        &[
            ("days", &days.to_string()),
            ("destination", &destination),
            ("schedule", &rendered),
            ("feasibility", &feasibility),
            ("cache_note", &limits.note),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "planning",
        "response:planning",
        &body,
        0.6,
    ))
}
