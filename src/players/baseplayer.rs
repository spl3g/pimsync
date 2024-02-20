use crate::piped_api::{Piped, PipedError, Video, Playlist};

#[derive(Debug)]
pub enum PlayerError {
    PipedError(PipedError),
    SqlxError(sqlx::Error),
    PlaylistNoVideos,
    PlaylistNoName,
}


pub trait Player {
    fn create_playlist(&self, playlist: &Playlist) -> Result<(), PlayerError>;
    fn delete_playlist(&self, name: String) -> Result<(), PlayerError>;
    fn add_songs(&self, videos: &Vec<Video>, playlist: String) -> Result<(), PlayerError>;
    fn get_songs(&self, playlist: String) -> Result<Vec<Video>, PlayerError>;
    fn get_playlists(&self) -> Result<Vec<Playlist>, PlayerError>;
    fn save(&self) -> Result<(), PlayerError>;
    fn connect(&self, db: String) -> Result<(), PlayerError>;
    fn import_from_piped(&self, piped: &Piped) -> Result<(), PlayerError> {
        let piped_playlists = piped.get_playlists()
            .map_err(|e| PlayerError::PipedError(e))?;
        let local_playlists = self.get_playlists()?;

        for playlist in piped_playlists.iter() {
            if !local_playlists.contains(playlist) {
                self.create_playlist(playlist);
            } else {
                self.add_songs(
                    playlist.videos
                        .as_ref()
                        .ok_or(PlayerError::PlaylistNoVideos)?,
                    playlist.name
                        .as_ref()
                        .ok_or(PlayerError::PlaylistNoName)?
                        .to_string()
                );
            }
        }
        Ok(())
    }
    fn export_to_piped(&self, piped: &Piped) -> Result<(), PlayerError> {
        let piped_playlists = piped.get_playlists()
            .map_err(|e| PlayerError::PipedError(e))?;
        let mut local_playlists = self.get_playlists()?;

        for playlist in local_playlists.iter_mut() {
            let playlist_name = playlist.name
                .as_ref()
                .ok_or(PlayerError::PlaylistNoName)?;

            if !piped_playlists.contains(playlist) {
                piped.create_playlist(playlist_name.clone()).map_err(|e| PlayerError::PipedError(e))?;

                let playlist_videos = playlist.videos
                    .as_mut()
                    .ok_or(PlayerError::PlaylistNoVideos)?
                    .iter_mut()
                    .map(|v| v.url.to_owned())
                    .collect();

                piped.add_videos(
                    playlist_name.clone(),
                    playlist_videos,
                );
            } else {
                let piped_videos = piped
                    .get_videos(playlist_name.clone())
                    .map_err(|e| PlayerError::PipedError(e))?;

                let playlist_videos = playlist.videos
                    .as_mut()
                    .ok_or(PlayerError::PlaylistNoVideos)?
                    .iter_mut()
                    .filter(|v| !piped_videos.contains(v))
                    .map(|v| v.url.to_owned())
                    .collect();
                
                piped.add_videos(
                    playlist_name.clone(),
                    playlist_videos,
                ).map_err(|e| PlayerError::PipedError(e));
            }
        }
        Ok(())
    }
}
