use crate::{players::baseplayer::{Player, PlayerError}, piped_api::{Piped, Playlist, Video}};
use sqlx::{sqlite::SqlitePool, Pool, Row, Sqlite};
use futures_lite::future::block_on;

#[derive(Debug)]
pub struct Vimusic {
    pub playlists: Option<Vec<Playlist>>,
    pool: Pool<Sqlite>,
}

impl Vimusic {
    pub fn connect(db: String) -> Result<Vimusic, PlayerError> {
        let pool = block_on(SqlitePool::connect(&db))
            .map_err(|e| PlayerError::SqlxError(e))?;

        Ok(Vimusic {
            playlists: None,
            pool,
        })
    }

    pub fn get_playlists(&mut self) -> Result<(), PlayerError> {
        let pl_rows = block_on(sqlx::query("SELECT * FROM Playlist")
            .fetch_all(&self.pool))
            .map_err(|e| PlayerError::SqlxError(e))?;

        let mut pls = Vec::new();

        for row in pl_rows.iter() {
            let pl_id: i32 = row.try_get("id").map_err(|e| PlayerError::SqlxError(e))?;
            let v_rows = block_on(sqlx::query(r#"SELECT * FROM SongPlaylistMap sp
left join Song s on sp.songID = s.id
where sp.playlistId = ?"#)
                .bind(pl_id)
                .fetch_all(&self.pool))
                .map_err(|e| PlayerError::SqlxError(e))?;

            let mut videos = Vec::new();

            for v_row in v_rows.iter() {
                videos.push(
                    Video {
                        title: v_row.try_get("title").map_err(|e| PlayerError::SqlxError(e))?,
                        url: v_row.try_get("id").map_err(|e| PlayerError::SqlxError(e))?,
                        uploader: v_row.try_get("artistsText").map_err(|e| PlayerError::SqlxError(e))?,
                    }
                )
            }

            pls.push(
                Playlist {
                    id: None,
                    videos: Some(videos),
                    name: row.try_get("name").map_err(|e| PlayerError::SqlxError(e))?,
                    url: match row.try_get("url") {
                        Ok(u) => Some(u),
                        Err(_) => None,
                    },
                }
            );
        };

        self.playlists = Some(pls);
        Ok(())
    }

    fn export_new(piped: &Piped, playlist: &mut Playlist) -> Result<(), PlayerError> {
        let playlist_name = playlist.name
            .as_ref()
            .ok_or(PlayerError::PlaylistNoName)?;

        let id = piped.create_playlist(playlist_name.clone())
            .map_err(|e| PlayerError::PipedError(e))?
            .id
            // Should not get here
            .unwrap();

        let playlist_videos = playlist.videos
            .as_mut()
            .ok_or(PlayerError::PlaylistNoVideos)?
            .iter_mut()
            .map(|v| v.url.to_owned())
            .collect();
        println!("{:?}", playlist_videos);

        let _ = piped.add_videos(
            id,
            playlist_videos,
        );

        Ok(())
    }

    fn export_existing(piped: &Piped, playlist: &mut Playlist) -> Result<(), PlayerError> {
        todo!();
        
        // let piped_videos: Vec<String> = piped
        //     .get_videos(playlist_name.clone())
        //     .map_err(|e| PlayerError::PipedError(e))?
        //     .iter_mut()
        //     .map(|v| v.url[9..].to_string())
        //     .collect();

        // let playlist_videos = playlist.videos
        //     .as_mut()
        //     .ok_or(PlayerError::PlaylistNoVideos)?
        //     .iter_mut()
        //     .map(|v| v.url.to_owned())
        //     .filter(|v| !piped_videos.contains(v))
        //     .collect();
                
        // let _ = piped.add_videos(
        //     playlist_name.clone(),
        //     playlist_videos,
        // ).map_err(|e| PlayerError::PipedError(e));
    }

    pub fn export_to_piped(&mut self, piped: &Piped) -> Result<(), PlayerError> {
        let piped_playlists = piped.get_playlists()
            .map_err(|e| PlayerError::PipedError(e))?;
        self.get_playlists()?;
        let mut local_playlists = self.playlists.as_mut().ok_or(PlayerError::PlaylistNoVideos)?;

        for playlist in local_playlists.iter_mut() {

            if !piped_playlists.iter().any(|p| p.name == playlist.name) {
                let _ = Vimusic::export_new(piped, playlist)?;
            } else {
                todo!();
            }
        }
        Ok(())
    }
}
