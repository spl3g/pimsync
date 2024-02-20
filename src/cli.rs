use clap::{Subcommand, Parser, Error};
use std::path::PathBuf;
use std::io;
use crate::piped_api::Playlist;
use std::collections::HashSet;


#[derive(Parser)]
pub struct Pimsync {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// player operations
    Sync {
        #[command(subcommand)]
        player: Option<Players>,
    },

    /// log in to a piped account
    Login {
        name: String,
        password: String,
        instance: String,
    },

    /// log out of a piped account
    Logout,
}

#[derive(Subcommand)]
enum Players {
    /// use vimusic db
    Vimusic {
        db: PathBuf,
    },
}

pub fn parse() {
    let args = Pimsync::parse();

    match args.command {
        Some(Commands::Sync { player }) => {
            match player {
                Some(Players::Vimusic { db }) => todo!(),
                None => todo!(),
            }
        },

        Some(Commands::Login { name, password, instance }) => {
            println!("You logged in");
        },

        Some(Commands::Logout) => todo!(),
        
        None => todo!(),
    }
}

fn list_playlists(playlists: Vec<Playlist>, action: String) -> Option<Vec<Playlist>> {
    println!("Choose playlists to {action}");
    for playlist in playlists.iter().enumerate() {
        println!("{}: {}", playlist.0, playlist.1.name.as_ref().unwrap_or(&"nill".to_string()));
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    if input.len() == 0 {
        return None;
    }

    let mut choice = HashSet::new();

    for el in input.split_whitespace() {
        if let Ok(num) = el.parse::<usize>() {
            if let Some(playlist) = playlists.get(num) {
                choice.insert(playlist);
            } else if el.contains('-') {
                let range: Vec<&str> = el.split('-').collect();
                if range.len() == 2 {
                    if let (Ok(start), Ok(end)) = (range[0].parse::<usize>(), range[1].parse::<usize>()) {
                        for i in start..=end {
                            if let Some(playlist) = playlists.get(i) {
                                choice.insert(playlist);
                            }
                    }
                    }
                }
            } else if el == "specific_word" {
                println!("Found the specific word: {}", el);
            }
        }
    }
    Some(Vec::new())
}
