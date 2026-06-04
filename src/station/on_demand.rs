const ORG_ID: &str = "2c1abd74-5b16-424c-97e9-ab91017281b8";
const PROG_ID: &str = "a9a55835-0db6-4372-a74f-ab9200b36b51";

#[derive(Debug, Clone)]
pub struct OnDemandShow {
    pub id: String,
    pub title: String,
    pub audio_url: String,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub playlist_id: &'static str,
}

pub const PROGRAMS: &[Program] = &[
    Program {
        playlist_id: "f0c7cbf9-f960-4699-a2ce-b2cf00cc72e0",
    },
    Program {
        playlist_id: "aaceb705-cd1e-41f3-a85b-ab940118a2f3",
    },
    Program {
        playlist_id: "b48deb81-e6a4-48bd-8aea-b24a01041176",
    },
    Program {
        playlist_id: "1f74745a-1bab-4b8b-ac33-b13000c1361f",
    },
    Program {
        playlist_id: "9f33aa7a-bab4-4ed6-8c66-b24a01080ea2",
    },
    Program {
        playlist_id: "4e6aa2c0-2965-4ecc-a197-b25f00f92143",
    },
    Program {
        playlist_id: "1dafe027-d54f-4c7b-b74e-b13000b369c1",
    },
    Program {
        playlist_id: "751c95db-f247-4b5e-88fa-b13000c058d2",
    },
    Program {
        playlist_id: "f7ec5a45-419f-4f5e-9697-b24800f94364",
    },
    Program {
        playlist_id: "a9a28357-b15d-4ca6-8a70-b13000b22504",
    },
    Program {
        playlist_id: "72259ef2-bd24-438a-841d-b13000bfe645",
    },
    Program {
        playlist_id: "cdc5213e-0814-48ca-9b29-b13000ad8ead",
    },
    Program {
        playlist_id: "632d3f71-395c-40c5-8158-b13000b28584",
    },
    Program {
        playlist_id: "0d6a3109-e41e-4dda-a4c7-b25900df74e1",
    },
    Program {
        playlist_id: "cbae7886-0569-4e2a-9c0c-b13000b30e21",
    },
    Program {
        playlist_id: "0e4c325e-5ca4-447e-bb42-b12c00e338d7",
    },
    Program {
        playlist_id: "69017bc7-9532-488f-b248-b13000c0c7f3",
    },
    Program {
        playlist_id: "4fe45e33-122d-48ee-9c5c-b2b5009e1be1",
    },
    Program {
        playlist_id: "e28ddf17-7553-4e60-91d5-b13000acb4fb",
    },
    Program {
        playlist_id: "7b11949f-bed5-45b1-85a9-b13000a3041a",
    },
];

pub fn audio_url(clip_id: &str) -> String {
    format!("https://traffic.omny.fm/d/clips/{ORG_ID}/{PROG_ID}/{clip_id}/audio.mp3")
}

pub async fn fetch_shows_for_playlist(playlist_id: &str) -> Option<Vec<OnDemandShow>> {
    let url =
        format!("https://api.omny.fm/orgs/{ORG_ID}/playlists/{playlist_id}/clips/v2?pageSize=20");

    let client = crate::http::http_client_timeout(15)?;

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        tracing::warn!(
            "Omny API HTTP {} para playlist {}",
            resp.status(),
            playlist_id
        );
        return None;
    }

    let body = resp.text().await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let clips = json["Clips"].as_array()?;

    let shows = clips
        .iter()
        .filter_map(|c| {
            let id = c["Id"].as_str()?.to_string();
            let title = c["Title"].as_str()?.to_string();
            let clip_audio_url = c["AudioUrl"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| audio_url(&id));

            Some(OnDemandShow {
                id,
                title,
                audio_url: clip_audio_url,
            })
        })
        .collect();

    tracing::info!(
        "Omny API: {} episodios cargados para playlist {}",
        clips.len(),
        playlist_id
    );
    Some(shows)
}
