use serde::Deserialize;
use serde_json::Value;

/// One movie, already narrowed to the media used for edition text.
#[derive(Debug, Clone)]
pub struct Movie {
    pub rating_key: String,
    pub title: String,
    pub library: String,
    pub edition_title: Option<String>,
    pub file_name: String,
    pub embedded_title: Option<String>,
    pub video_resolution: Option<String>,
    pub video_codec: Option<String>,
    pub video_frame_rate: Option<String>,
    pub video_display_title: Option<String>,
    pub dovi_profile: Option<Value>,
    pub audio_display_title: Option<String>,
    pub bitrate: Option<f64>,
    pub file_size: u64,
    pub countries: Vec<String>,
    pub content_rating: Option<String>,
    pub audience_rating: Option<Value>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    #[serde(rename = "MediaContainer")]
    container: Value,
}

pub fn friendly_name(body: &Value) -> Option<String> {
    body.get("MediaContainer")
        .and_then(|container| container.get("friendlyName"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub fn libraries(body: &Value) -> Vec<Library> {
    body.get("MediaContainer")
        .and_then(|container| container.get("Directory"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let title = entry.get("title")?.as_str()?.to_string();
                    let key = entry.get("key")?.as_str()?.to_string();
                    let kind = entry.get("type")?.as_str()?.to_string();
                    Some(Library { title, key, kind })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct Library {
    pub title: String,
    pub key: String,
    pub kind: String,
}

pub fn movies_from_section(body: &Value, library: &str) -> Vec<Movie> {
    metadata_list(body)
        .into_iter()
        .filter_map(|item| movie_from_item(item, library))
        .collect()
}

pub fn movie_from_metadata(body: &Value, library: &str) -> Option<Movie> {
    metadata_list(body)
        .into_iter()
        .find_map(|item| movie_from_item(item, library))
}

fn metadata_list(body: &Value) -> Vec<&Value> {
    body.get("MediaContainer")
        .and_then(|container| container.get("Metadata"))
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn movie_from_item(item: &Value, library: &str) -> Option<Movie> {
    if item.get("type").and_then(Value::as_str) != Some("movie") {
        return None;
    }
    let rating_key = json_string(item, "ratingKey")?;
    let title = json_string(item, "title").unwrap_or_else(|| rating_key.clone());
    let edition_title = json_string(item, "editionTitle").filter(|value| !value.is_empty());
    let part = largest_part(item);
    let file_name = part.file_name;
    let file_size = part.file_size;
    let bitrate = part.bitrate;
    let video_resolution = part.video_resolution;
    let video_codec = part.video_codec;
    let video_frame_rate = part.video_frame_rate;
    let duration_ms = part.duration_ms;
    let streams = video_streams_of_largest(item);
    let embedded_title = streams
        .as_ref()
        .and_then(|list| first_stream(list, 1))
        .and_then(|stream| json_string(stream, "title"));
    let video = streams.as_ref().and_then(|list| first_stream(list, 1));
    let video_display_title = video.and_then(|stream| json_string(stream, "displayTitle"));
    let dovi_profile = video.and_then(|stream| stream.get("DOVIProfile").cloned());
    let audio_display_title = streams
        .as_ref()
        .and_then(|list| best_audio(list))
        .and_then(|stream| json_string(stream, "displayTitle"));
    let countries = item
        .get("Country")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| json_string(entry, "tag"))
                .collect()
        })
        .unwrap_or_default();
    Some(Movie {
        rating_key,
        title,
        library: library.to_string(),
        edition_title,
        file_name,
        embedded_title,
        video_resolution,
        video_codec,
        video_frame_rate,
        video_display_title,
        dovi_profile,
        audio_display_title,
        bitrate,
        file_size,
        countries,
        content_rating: json_string(item, "contentRating"),
        audience_rating: item.get("audienceRating").cloned(),
        duration_ms,
    })
}

struct LargestPart {
    file_name: String,
    file_size: u64,
    bitrate: Option<f64>,
    video_resolution: Option<String>,
    video_codec: Option<String>,
    video_frame_rate: Option<String>,
    duration_ms: Option<u64>,
}

/// Largest part across every Media, matching the size scan used for bitrate and size.
fn largest_part(item: &Value) -> LargestPart {
    let mut best_size = 0_u64;
    let mut best: Option<&Value> = None;
    let mut best_media: Option<&Value> = None;
    for media in item
        .get("Media")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for part in media
            .get("Part")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let size = part.get("size").and_then(Value::as_u64).unwrap_or(0);
            if best.is_none() || size > best_size {
                best_size = size;
                best = Some(part);
                best_media = Some(media);
            }
        }
    }
    let file_name = best
        .and_then(|part| json_string(part, "file"))
        .map(|path| {
            path.rsplit(['/', '\\'])
                .next()
                .unwrap_or(path.as_str())
                .to_string()
        })
        .unwrap_or_default();
    let media = best_media;
    LargestPart {
        file_name,
        file_size: best_size,
        bitrate: media
            .and_then(|media| media.get("bitrate"))
            .and_then(as_f64),
        video_resolution: media.and_then(|media| json_string(media, "videoResolution")),
        video_codec: media.and_then(|media| json_string(media, "videoCodec")),
        video_frame_rate: media.and_then(|media| json_string(media, "videoFrameRate")),
        duration_ms: media
            .and_then(|media| media.get("duration"))
            .and_then(Value::as_u64),
    }
}

fn video_streams_of_largest(item: &Value) -> Option<Vec<Value>> {
    let mut best_size = 0_u64;
    let mut best: Option<&Value> = None;
    for media in item
        .get("Media")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        for part in media
            .get("Part")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let size = part.get("size").and_then(Value::as_u64).unwrap_or(0);
            if best.is_none() || size > best_size {
                best_size = size;
                best = Some(part);
            }
        }
    }
    best.and_then(|part| part.get("Stream"))
        .and_then(Value::as_array)
        .cloned()
}

fn first_stream(streams: &[Value], stream_type: i64) -> Option<&Value> {
    streams
        .iter()
        .find(|stream| stream.get("streamType").and_then(Value::as_i64) == Some(stream_type))
}

fn best_audio(streams: &[Value]) -> Option<&Value> {
    streams
        .iter()
        .filter(|stream| stream.get("streamType").and_then(Value::as_i64) == Some(2))
        .max_by(|left, right| {
            let channels =
                |stream: &Value| stream.get("channels").and_then(Value::as_i64).unwrap_or(0);
            let bitrate =
                |stream: &Value| stream.get("bitrate").and_then(Value::as_i64).unwrap_or(0);
            (channels(left), bitrate(left)).cmp(&(channels(right), bitrate(right)))
        })
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    match value.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn as_f64(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|number| number as f64))
        .or_else(|| value.as_u64().map(|number| number as f64))
}

/// Decode a Plex JSON document. A non-object body is a decode error.
pub fn parse_body(bytes: &[u8]) -> crate::error::Result<Value> {
    serde_json::from_slice(bytes).map_err(|err| crate::error::Error::Decode(err.to_string()))
}

#[allow(dead_code)]
fn _envelope_used(body: Envelope) -> Value {
    body.container
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_largest_part_across_media() {
        let body = serde_json::json!({
            "MediaContainer": {
                "Metadata": [{
                    "type": "movie",
                    "ratingKey": "9",
                    "title": "Example",
                    "Media": [
                        {
                            "videoResolution": "1080",
                            "bitrate": 1000,
                            "duration": 60000,
                            "Part": [{ "file": "/a/small.mkv", "size": 10 }]
                        },
                        {
                            "videoResolution": "4k",
                            "videoCodec": "hevc",
                            "bitrate": 8000,
                            "duration": 120000,
                            "Part": [{
                                "file": "/b/big.mkv",
                                "size": 99,
                                "Stream": [{ "streamType": 1, "title": "HEVC", "displayTitle": "4K DoVi" }]
                            }]
                        }
                    ]
                }]
            }
        });
        let movie = movies_from_section(&body, "Movies").pop().unwrap();
        assert_eq!(movie.file_name, "big.mkv");
        assert_eq!(movie.file_size, 99);
        assert_eq!(movie.video_resolution.as_deref(), Some("4k"));
        assert_eq!(movie.bitrate, Some(8000.0));
        assert_eq!(movie.embedded_title.as_deref(), Some("HEVC"));
    }
}
