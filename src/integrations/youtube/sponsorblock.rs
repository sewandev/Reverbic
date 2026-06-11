use serde::Deserialize;

const SPONSORBLOCK_API: &str = "https://sponsor.ajay.app/api/skipSegments";

#[derive(Deserialize)]
struct SkipSegment {
    segment: [f32; 2],
}

pub async fn fetch_music_offtopic_segments(video_id: &str) -> Result<Vec<(f32, f32)>, String> {
    if video_id.is_empty()
        || video_id.len() > 16
        || !video_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("invalid video id".to_string());
    }

    let client = crate::http::http_client_timeout(10).ok_or("could not build HTTP client")?;
    let url = format!("{SPONSORBLOCK_API}?videoID={video_id}&categories=[\"music_offtopic\"]");
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    // 404 means the video simply has no submitted segments
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }
    let segments: Vec<SkipSegment> = resp
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(segments
        .into_iter()
        .map(|s| (s.segment[0], s.segment[1]))
        .filter(|(start, end)| end > start && *start >= 0.0)
        .collect())
}
