pub fn content_rating(raw: Option<&str>) -> Option<String> {
    let rating = raw?;
    if rating == "Not Rated" {
        Some("NR".into())
    } else {
        Some(rating.to_string())
    }
}
