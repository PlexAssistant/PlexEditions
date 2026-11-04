mod audio;
mod bitrate;
mod content_rating;
mod country;
mod cut;
mod duration;
mod dynamic_range;
mod frame_rate;
mod rating;
mod release;
mod resolution;
mod size;
mod source;
mod video;

use regex::Regex;
use std::sync::OnceLock;

use crate::config::Language;
use crate::model::Movie;

pub fn render(movie: &Movie, modules: &[String], language: Language, separator: &str) -> String {
    modules
        .iter()
        .filter_map(|module| part(module, movie, language))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

fn part(module: &str, movie: &Movie, language: Language) -> Option<String> {
    match module {
        "source" => source::source(&movie.file_name, movie.embedded_title.as_deref()),
        "cut" => cut::cut(&movie.file_name, movie.embedded_title.as_deref(), language),
        "release" => release::release(&movie.file_name, movie.embedded_title.as_deref(), language),
        "resolution" => resolution::resolution(movie.video_resolution.as_deref()),
        "dynamicrange" => dynamic_range::dynamic_range(
            movie.video_display_title.as_deref(),
            movie.dovi_profile.as_ref(),
        ),
        "videocodec" => video::video_codec(movie.video_codec.as_deref()),
        "framerate" => frame_rate::frame_rate(movie.video_frame_rate.as_deref()),
        "audiocodec" => audio::audio_codec(movie.audio_display_title.as_deref(), language),
        "bitrate" => bitrate::bitrate(movie.bitrate),
        "size" => size::size(movie.file_size),
        "country" => country::country(&movie.countries, language),
        "contentrating" => content_rating::content_rating(movie.content_rating.as_deref()),
        "rating" => rating::rating(movie.audience_rating.as_ref()),
        "duration" => duration::duration(movie.duration_ms, language),
        _ => None,
    }
}

fn search(pattern: &'static str, text: &str) -> bool {
    regex(pattern).is_match(text)
}

fn regex(pattern: &'static str) -> &'static Regex {
    use std::collections::HashMap;
    static CACHE: OnceLock<std::sync::Mutex<HashMap<&'static str, &'static Regex>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut cache = cache.lock().expect("regex cache");
    if let Some(compiled) = cache.get(pattern) {
        return compiled;
    }
    let compiled: &'static Regex =
        Box::leak(Box::new(Regex::new(pattern).expect("module pattern")));
    cache.insert(pattern, compiled);
    compiled
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn movie() -> Movie {
        Movie {
            rating_key: "1".into(),
            title: "Example".into(),
            library: "Movies".into(),
            edition_title: None,
            file_name: "Example.1982.REMUX.2160p.mkv".into(),
            embedded_title: None,
            video_resolution: Some("4k".into()),
            video_codec: Some("hevc".into()),
            video_frame_rate: Some("24p".into()),
            video_display_title: Some("4K HDR10 (HEVC)".into()),
            dovi_profile: None,
            audio_display_title: Some("English (TRUEHD 7.1)".into()),
            bitrate: Some(8000.0),
            file_size: 20 * 1024 * 1024 * 1024,
            countries: vec!["United States".into()],
            content_rating: Some("PG-13".into()),
            audience_rating: Some(json!(8.2)),
            duration_ms: Some(120 * 60 * 1000),
        }
    }

    #[test]
    fn joins_selected_modules_and_skips_blanks() {
        let text = render(
            &movie(),
            &["source".into(), "cut".into(), "resolution".into()],
            Language::En,
            " · ",
        );
        assert_eq!(text, "REMUX · 4K");
    }
}
