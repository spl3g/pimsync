use reqwest::{self, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum PipedError {
    WrongCreds,
    PlaylistNoVideos,
    ReqwestError(Error),
}

#[derive(Deserialize, Debug)]
struct Token {
    token: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Video {
    pub title: String,
    pub url: String,
    #[serde(rename = "uploaderName")]
    pub uploader: String,
} 

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug, Clone)]
pub struct Playlist {
    #[serde(skip_serializing, alias = "playlistId")]
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(skip_serializing, rename = "relatedStreams")]
    pub videos: Option<Vec<Video>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub url: Option<String>,
}

#[derive(Serialize)]
struct LoginCreds {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AddVideosReq {
    #[serde(rename = "playlistId")]
    playlist_id: String,
    #[serde(rename = "videoIds")]
    video_ids: Vec<String>,
}

#[derive(Debug)]
pub struct Piped {
    token: String,
    pub instance: String,
    client: reqwest::blocking::Client,
}

impl Piped {
    pub fn get_instances() -> Result<Vec<Vec<String>>, Error> {
        let res = reqwest::blocking::get("https://raw.githubusercontent.com/wiki/TeamPiped/Piped-Frontend/Instances.md")?
            .text()?;

        let instances: Vec<Vec<String>> = res.lines()
            .skip(8)
            .map(|l| l.split('|').map(|s| s.trim().to_string()).collect())
            .collect();

        Ok(instances)
}

    pub fn auth(username: String, password: String, instance: String) -> Result<Self, PipedError> {
        let user_agent = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        );

        
        let client = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .build()
            .map_err(|e| PipedError::ReqwestError(e))?;

        let creds = LoginCreds {
            username: username.to_string(),
            password: password.to_string(),
        };

        let res = client.post(instance.to_owned() + "login")
            .json(&creds)
            .send()
            .map_err(|e| PipedError::ReqwestError(e))?
            .json::<Token>()
            .map_err(|e| PipedError::ReqwestError(e))?;
        
        if res.error.is_some() {
            Err(PipedError::WrongCreds)
        } else {
            Ok(Piped {
                token: res.token.unwrap(),
                client,
                instance: instance.to_string(),
            })
        }
    }

    pub fn create_playlist(&self, name: String) -> Result<Playlist, PipedError> {
        let pl = Playlist {
            id: None,
            name: Some(name.clone()),
            videos: None,
            url: None,
        }; 
        
        let mut post = self.client.post(self.instance.clone() + "user/playlists/create")
            .header("authorization", &self.token)
            .json(&pl)
            .send()
            .map_err(|e| PipedError::ReqwestError(e))?
            .json::<Playlist>()
            .map_err(|e| PipedError::ReqwestError(e))?;
        post.name = Some(name);

        println!("{:?}", &post);
        Ok(post)
    }
    
    pub fn get_playlists(&self) -> Result<Vec<Playlist>, PipedError> {
        Ok(self.client.get(self.instance.clone() + "user/playlists")
            .header("Authorization", &self.token)
            .send()
            .map_err(|e| PipedError::ReqwestError(e))?
            .json::<Vec<Playlist>>()
            .map_err(|e| PipedError::ReqwestError(e))?)
    }
    pub fn add_videos(&self, playlist_id: String, video_ids: Vec<String>) -> Result<(), PipedError> {
        let req = AddVideosReq {
            playlist_id,
            video_ids,
        };
        
        self.client.post(self.instance.clone() + "user/playlists/add")
            .header("Authorization", &self.token)
            .json(&req)
            .send()
            .map_err(|e| PipedError::ReqwestError(e))?;

        Ok(())
    }

    pub fn get_videos(&self, playlist: String) -> Result<Vec<Video>, PipedError> {
        let playlist = self.client.get(self.instance.clone() + "playlists/" + &playlist)
            .send()
            .map_err(|e| PipedError::ReqwestError(e))?
            .json::<Playlist>()
            .map_err(|e| PipedError::ReqwestError(e))?;

        Ok(playlist.videos.ok_or(PipedError::PlaylistNoVideos)?)
    }
}
