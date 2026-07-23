use serde::{Deserialize, Serialize};
use axum::{
    extract::{Path, Request},
    response::IntoResponse,
    routing::get,
    Router,
    http::{StatusCode, HeaderMap, HeaderValue},
};
use reqwest::header::{RANGE, CONTENT_TYPE, CONTENT_LENGTH, ACCEPT_RANGES, CONTENT_RANGE};
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub thumbnail: String,
    pub duration: String,
}

fn parse_yt_music_search(json: &serde_json::Value) -> Vec<Song> {
    let mut songs = Vec::new();

    let contents = json
        .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|v| v.as_array());

    if let Some(sections) = contents {
        for section in sections {
            let items = section
                .pointer("/musicShelfRenderer/contents")
                .or_else(|| section.pointer("/musicCardShelfRenderer/contents"))
                .and_then(|v| v.as_array());

            if let Some(items_list) = items {
                for item in items_list {
                    if let Some(renderer) = item.pointer("/musicResponsiveListItemRenderer") {
                        let id = renderer
                            .pointer("/playlistItemData/videoId")
                            .or_else(|| renderer.pointer("/doubleTapCommand/watchEndpoint/videoId"))
                            .or_else(|| renderer.pointer("/overlay/musicItemHoverOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId"))
                            .and_then(|v| v.as_str());

                        if let Some(video_id) = id {
                            let title = renderer
                                .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Sem título");

                            let thumbnail = renderer
                                .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails/0/url")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            let mut artist = String::from("Artista Desconhecido");
                            let mut duration = String::from("--:--");

                            if let Some(runs) = renderer
                                .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs")
                                .and_then(|v| v.as_array())
                            {
                                let texts: Vec<&str> = runs
                                    .iter()
                                    .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                                    .collect();

                                let filtered: Vec<&&str> = texts
                                    .iter()
                                    .filter(|&&t| t != " • " && t != "Música" && t != "Song" && t != "Canção" && t != "Vídeo")
                                    .collect();

                                if !filtered.is_empty() {
                                    artist = filtered[0].to_string();
                                }
                                if filtered.len() > 1 {
                                    if let Some(last) = filtered.last() {
                                        if last.contains(':') {
                                            duration = last.to_string();
                                        }
                                    }
                                }
                            }

                            songs.push(Song {
                                id: video_id.to_string(),
                                title: title.to_string(),
                                artist,
                                thumbnail: thumbnail.to_string(),
                                duration,
                            });
                        }
                    }
                }
            }
        }
    }

    songs
}

#[tauri::command]
async fn search_songs(query: String) -> Result<Vec<Song>, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20231214.01.00",
                "hl": "pt",
                "gl": "BR"
            }
        },
        "query": query,
        "params": "EgWKAQIIAWoKEAkQBRAKEAMQBA=="
    });

    let res = client
        .post("https://music.youtube.com/youtubei/v1/search")
        .json(&body)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(parse_yt_music_search(&json))
}

async fn fetch_direct_audio_url(video_id: &str) -> Result<String, String> {
    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

    let output = Command::new("yt-dlp")
        .args([
            "-g",                   
            "-f", "ba/b",          
            &video_url,
        ])
        .output()
        .await
        .map_err(|e| format!("Falha ao executar yt-dlp: {}. Verifique se o pacote 'yt-dlp' está instalado no Arch.", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp erro: {}", err_msg));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        return Err("yt-dlp retornou uma URL vazia.".to_string());
    }

    Ok(url)
}

async fn handle_stream(
    Path(video_id): Path<String>,
    req: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!("[STREAM] Solicitando áudio via yt-dlp para o vídeo ID: {}", video_id);

    let audio_url = fetch_direct_audio_url(&video_id).await.map_err(|e| {
        eprintln!("[STREAM ERRO] Extrator falhou: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

    println!("[STREAM] URL direta obtida com sucesso do yt-dlp!");

    let client = reqwest::Client::new();
    let mut yt_req = client
        .get(&audio_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)");

    if let Some(range) = req.headers().get(RANGE) {
        yt_req = yt_req.header(RANGE, range);
    }

    let yt_res = yt_req.send().await.map_err(|e| {
        eprintln!("[STREAM ERRO] Falha ao conectar nos servidores de mídia da Google: {}", e);
        (StatusCode::BAD_GATEWAY, e.to_string())
    })?;

    let status = yt_res.status();
    println!("[STREAM] Status da resposta do áudio: {}", status);

    let mut response_headers = HeaderMap::new();

    if let Some(ct) = yt_res.headers().get(CONTENT_TYPE) {
        response_headers.insert(CONTENT_TYPE, ct.clone());
    }
    if let Some(cl) = yt_res.headers().get(CONTENT_LENGTH) {
        response_headers.insert(CONTENT_LENGTH, cl.clone());
    }
    if let Some(cr) = yt_res.headers().get(CONTENT_RANGE) {
        response_headers.insert(CONTENT_RANGE, cr.clone());
    }
    response_headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response_headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));

    let stream = yt_res.bytes_stream();
    let body = axum::body::Body::from_stream(stream);

    Ok((status, response_headers, body))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::spawn(async {
        let app = Router::new().route("/stream/:id", get(handle_stream));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9876")
            .await
            .expect("Falha ao iniciar servidor de proxy de áudio local");
        axum::serve(listener, app).await.unwrap();
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![search_songs])
        .run(tauri::generate_context!())
        .expect("erro ao rodar o app tauri");
}