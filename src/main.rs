use std::io;
use reqwest;
use reqwest::{Request, Method, Url, StatusCode};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use termion::{color, style};

#[derive(Deserialize, Clone)]
struct GithubFollowerInfo {
    login: String,
    id: u64,
}

async fn check_is_follow(username: String, somebody: String) -> Result<StatusCode, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/users/{somebody}/following/{my}",
                      my = username,
                      somebody = somebody);

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::http("http://127.0.0.1:1087/")?)
        .build()?;

    let mut request = Request::new(Method::GET, Url::parse(url.as_str()).unwrap());
    request.headers_mut().append(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    request.headers_mut().append(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4147.125 Safari/537.36".parse().unwrap());

    let status: StatusCode = client
        .execute(request)
        .await?
        .status();

    Ok(status)
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

    let mut page: i32 = 1;
    let mut followers: Vec<GithubFollowerInfo> = Vec::new();
    let mut current_followers: Vec<GithubFollowerInfo> = Vec::new();

    while page == 1 || current_followers.len() > 0 {
        println!("Scanning Follower Page = {}", page);

        let url = format!("https://api.github.com/users/{username}/following?per_page={per_page}&page={page}",
                          username = username,
                          per_page = 100,
                          page = page);

        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::http("http://127.0.0.1:1087/")?)
            .build()?;

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

    let total_followers: usize = followers.len();
    println!("People you are following ({}): \n", total_followers);
    let mut following: usize = 0;

    for item in followers {
        let somebody = item.login;
        let follow = check_is_follow(username.clone(), somebody.clone()).await?;

        if follow.as_u16() == 204 {
            following += 1;

            println!("{colorA}[  Following You] {colorB}{bold}{somebody} {clearColor}{clearStyle}(https://github.com/{somebody})",
                     somebody = somebody,
                     colorA = color::Fg(color::Green),
                     colorB = color::Fg(color::Blue),
                     bold = style::Bold,
                     clearStyle = style::Reset,
                     clearColor = color::Fg(color::Reset)
            );
        } else {
            println!("{colorA}[unFollowing You] {colorB}{bold}{somebody} {clearColor}{clearStyle}(https://github.com/{somebody})",
                     somebody = somebody,
                     colorA = color::Fg(color::Red),
                     colorB = color::Fg(color::Blue),
                     bold = style::Bold,
                     clearStyle = style::Reset,
                     clearColor = color::Fg(color::Reset)
            );
        }
    }

    println!("\nStatics: {} following you, {} not following you, {}% mutual following rate",
             following, total_followers - following, 100.0 * (following as f32) / (total_followers as f32));
    Ok(())
}
