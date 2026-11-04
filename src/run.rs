use std::time::Duration;

use tracing::info;

use crate::api::Client;
use crate::config::Config;
use crate::error::Result;
use crate::model::Movie;
use crate::modules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    All,
    Reset,
}

pub struct Stats {
    pub written: u64,
    pub skipped: u64,
    pub failed: u64,
}

pub async fn run(config: &Config, client: &Client, mode: Mode) -> Result<Stats> {
    let mut stats = Stats {
        written: 0,
        skipped: 0,
        failed: 0,
    };
    let libraries = client.libraries().await?;
    for library in libraries {
        if library.kind != "movie" || config.skip_libraries.contains(&library.title) {
            continue;
        }
        let movies = match client.movies(&library).await {
            Ok(movies) => movies,
            Err(err) => {
                info!(library = %library.title, error = %err, "library skipped");
                stats.failed += 1;
                continue;
            }
        };
        match mode {
            Mode::All => info!("processing {} ({} movies)", library.title, movies.len()),
            Mode::Reset => info!("resetting {} ({} movies)", library.title, movies.len()),
        }
        for movie in movies {
            match apply(config, client, mode, &movie).await {
                Ok(Action::Wrote) => stats.written += 1,
                Ok(Action::Skipped) => stats.skipped += 1,
                Err(err) => {
                    info!(movie = %movie.title, error = %err, "movie skipped");
                    stats.failed += 1;
                }
            }
        }
    }
    Ok(stats)
}

/// Process one movie by id. `Ok(true)` means no further attempt is needed:
/// an edition was written, one was already present, or the movie is out of
/// scope. `Ok(false)` means nothing could be composed yet, so the caller
/// may retry later.
pub async fn apply_one(config: &Config, client: &Client, rating_key: &str) -> Result<bool> {
    let Some(movie) = client.movie(rating_key).await? else {
        return Ok(true);
    };
    if !movie.library.is_empty() && config.skip_libraries.contains(&movie.library) {
        return Ok(true);
    }
    let action = apply(config, client, Mode::All, &movie).await?;
    Ok(action == Action::Wrote || movie.edition_title.is_some())
}

#[derive(PartialEq, Eq)]
enum Action {
    Wrote,
    Skipped,
}

async fn apply(config: &Config, client: &Client, mode: Mode, movie: &Movie) -> Result<Action> {
    match mode {
        Mode::All => write_if_empty(config, client, movie).await,
        Mode::Reset => clear_if_present(config, client, movie).await,
    }
}

async fn write_if_empty(config: &Config, client: &Client, movie: &Movie) -> Result<Action> {
    if movie.edition_title.is_some() {
        return Ok(Action::Skipped);
    }
    let edition = modules::render(movie, &config.modules, config.language, &config.separator);
    if edition.is_empty() {
        return Ok(Action::Skipped);
    }
    let line = format!("{}: {edition}", movie.title);
    if config.dry_run {
        info!("{line}");
        return Ok(Action::Wrote);
    }
    client.set_edition(&movie.rating_key, &edition).await?;
    info!("{line}");
    Ok(Action::Wrote)
}

async fn clear_if_present(config: &Config, client: &Client, movie: &Movie) -> Result<Action> {
    if movie.edition_title.is_none() {
        return Ok(Action::Skipped);
    }
    let line = format!("reset {}", movie.title);
    if config.dry_run {
        info!("{line}");
        return Ok(Action::Wrote);
    }
    client.clear_edition(&movie.rating_key).await?;
    info!("{line}");
    Ok(Action::Wrote)
}

pub fn backoff(attempt: u32) -> Duration {
    let shift = attempt.min(6);
    Duration::from_secs(1_u64 << shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Library;
    use std::collections::HashSet;

    #[test]
    fn skips_non_movie_and_named_libraries() {
        let libraries = [
            Library {
                title: "Movies".into(),
                key: "1".into(),
                kind: "movie".into(),
            },
            Library {
                title: "Shows".into(),
                key: "2".into(),
                kind: "show".into(),
            },
            Library {
                title: "演唱会".into(),
                key: "3".into(),
                kind: "movie".into(),
            },
        ];
        let skip = HashSet::from(["演唱会".to_string()]);
        let kept: Vec<_> = libraries
            .iter()
            .filter(|library| library.kind == "movie" && !skip.contains(&library.title))
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].title, "Movies");
    }

    #[test]
    fn backoff_doubles_then_caps() {
        assert_eq!(backoff(0), Duration::from_secs(1));
        assert_eq!(backoff(3), Duration::from_secs(8));
        assert_eq!(backoff(10), Duration::from_secs(64));
    }
}
