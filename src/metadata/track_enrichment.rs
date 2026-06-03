use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct EnrichedTrack {
    pub artist:        String,
    pub title:        String,
    pub album:        String,
    pub year:         Option<u16>,
    pub duration_secs: u32,
}

pub async fn enrich(icy_title: &str) -> Option<EnrichedTrack> {
    let (artist, title) = parse_icy(icy_title)?;
    let query = format!("{} {}", artist, title);

    try_deezer(&query, &artist, &title).await
        .or(try_itunes(&query, &artist, &title).await)
}

fn parse_icy(icy: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = icy.splitn(2, " - ").collect();
    match parts.as_slice() {
        [a, t] if !a.is_empty() && !t.is_empty() => {
            Some((a.trim().to_string(), t.trim().to_string()))
        }
        _ => None,
    }
}

async fn try_deezer(query: &str, artist: &str, title: &str) -> Option<EnrichedTrack> {
    let encoded = encode(query);
    let url = format!("https://api.deezer.com/search?q={encoded}&limit=1");
    let client = crate::http::http_client_timeout(8)?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    #[derive(Deserialize)]
    struct Root {
        data: Vec<DeezerTrack>,
    }
    #[derive(Deserialize)]
    struct DeezerTrack {
        title:    String,
        duration: u32,
        artist:   DeezerArtist,
        album:    DeezerAlbum,
    }
    #[derive(Deserialize)]
    struct DeezerArtist {
        name: String,
    }
    #[derive(Deserialize)]
    struct DeezerAlbum {
        title: String,
    }

    let root: Root = resp.json().await.ok()?;
    let track = root.data.into_iter().next()?;

    if !fuzzy_match(&track.title, title) || !fuzzy_match(&track.artist.name, artist) {
        return None;
    }

    Some(EnrichedTrack {
        artist:        track.artist.name,
        title:         track.title,
        album:         track.album.title,
        year:          None,
        duration_secs: track.duration,
    })
}

async fn try_itunes(query: &str, artist: &str, title: &str) -> Option<EnrichedTrack> {
    let encoded = encode(query);
    let url = format!(
        "https://itunes.apple.com/search?term={encoded}&entity=song&limit=1&media=music"
    );
    let client = crate::http::http_client_timeout(8)?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Root {
        results: Vec<ItunesTrack>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ItunesTrack {
        track_name:        String,
        artist_name:       String,
        collection_name:   String,
        release_date:      Option<String>,
        track_time_millis: Option<u64>,
    }

    let root: Root = resp.json().await.ok()?;
    let track = root.results.into_iter().next()?;

    if !fuzzy_match(&track.track_name, title) || !fuzzy_match(&track.artist_name, artist) {
        return None;
    }

    let year = track.release_date
        .as_deref()
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse().ok());
    let duration_secs = track.track_time_millis.unwrap_or(0) as u32 / 1000;

    Some(EnrichedTrack {
        artist:        track.artist_name,
        title:         track.track_name,
        album:         track.collection_name,
        year,
        duration_secs,
    })
}

fn fuzzy_match(a: &str, b: &str) -> bool {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    a.contains(b.as_str()) || b.contains(a.as_str())
}

fn encode(s: &str) -> String {
    s.bytes().fold(String::new(), |mut out, b| {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
        out
    })
}
