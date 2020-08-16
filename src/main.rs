use std::io;
use reqwest;
use reqwest::{Request, Method, Url};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct GithubFollowerInfo {
    login: String,
    id: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut username = String::new();
    println!("Input Github username:");
    match io::stdin().read_line(&mut username) {
        Err(e) => {
            println!("Error {}", e);
            return Ok(());
        }
        _ => {}
    }

    println!("Username: {}", username);
    println!("Connecting to Github Server...");

    let mut page = 1;
    let mut followers: Vec<GithubFollowerInfo> = Vec::new();
    let mut current_followers: Vec<GithubFollowerInfo> = Vec::new();

    while page == 1 || current_followers.len() > 0 {
        println!("Scanning Follower Page = {}", page);

        let url = format!("https://api.github.com/users/{username}/following?per_page={per_page}&page={page}",
                          username = username,
                          per_page = 100,
                          page = page);

        let client = reqwest::Client::new();
        let mut request = Request::new(Method::GET, Url::parse(url.as_str()).unwrap());
        request.headers_mut().append(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
        request.headers_mut().append(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4147.125 Safari/537.36".parse().unwrap());

        current_followers = client
            .execute(request)
            .await?
            .json()
            .await?;

        page += 1;
        for item in current_followers.iter() {
            followers.push((*item).clone());
        }

        if current_followers.len() < 100 {
            break;
        }
    }

    println!("People you are following ({}): \n", followers.len());
    for item in followers {
        println!("{} (https://github.com/{})", item.login, item.login);
    }

    Ok(())
}
