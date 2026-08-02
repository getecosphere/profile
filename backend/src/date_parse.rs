use chrono::NaiveDate;

/// Mirrors UserService.parseDate from the Java version: tries ISO
/// (YYYY-MM-DD) first, then DD/MM/YYYY, returns None (silently, same as
/// the Java version logged-and-returned-null) if neither matches.
pub fn parse_date(input: &Option<String>) -> Option<bson::DateTime> {
    let s = input.as_ref()?;
    if s.is_empty() {
        return None;
    }

    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return to_bson(date);
    }

    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        if let (Ok(day), Ok(month), Ok(year)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<i32>(),
        ) {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                return to_bson(date);
            }
        }
    }

    tracing::warn!(value = %s, "failed to parse date");
    None
}

fn to_bson(date: NaiveDate) -> Option<bson::DateTime> {
    let datetime = date.and_hms_opt(0, 0, 0)?;
    Some(bson::DateTime::from_chrono(
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(datetime, chrono::Utc),
    ))
}
