use std::io;
use reqwest;
use reqwest::{Request, Method, Url, StatusCode, Client};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use termion::{color, style};

const USE_PROXY: bool = true;

fn create_client() -> Client {
    if USE_PROXY {
        return reqwest::Client::builder()
            .proxy(reqwest::Proxy::http("http://127.0.0.1:1087/").unwrap())
            .build().unwrap();
    } else {
        return reqwest::Client::new();
    }
}

#[derive(Deserialize, Clone)]
struct GithubFollowerInfo {
    login: String,
    id: u64,
}

async fn check_is_follow(username: String, somebody: String) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/users/{somebody}/following/{my}",
                      my = username,
                      somebody = somebody);

    let client = create_client();
    let mut request = Request::new(Method::GET, Url::parse(url.as_str()).unwrap());
    request.headers_mut().append(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    request.headers_mut().append(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/84.0.4147.125 Safari/537.36".parse().unwrap());

    let resp = client
        .execute(request)
        .await?;

    let status: StatusCode = resp.status();
    if status.as_u16() == 403 {
        let resp_text = resp.text().await?;
        println!("{}", resp_text);
    }

    Ok((status, somebody))
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

        let client = create_client();
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

    let mut all_futures = Vec::new();
    for item in followers {
        let somebody = item.login;

        let follow_future = check_is_follow(username.clone(), somebody.clone());
        all_futures.push(follow_future);
    }

    let all_futures = futures::future::join_all(all_futures);
    let mut error_count: usize = 0;
    for item in all_futures.await.iter() {
        match item {
            Ok(res) => {
                let (status, somebody) = res;

                if status.as_u16() == 204 {
                    following += 1;

                    println!("{colorA}[  Following You] {colorB}{bold}{somebody}{clearColor}{clearStyle} (https://github.com/{somebody})",
                             somebody = somebody,
                             colorA = color::Fg(color::Green),
                             colorB = color::Fg(color::Blue),
                             bold = style::Bold,
                             clearStyle = style::Reset,
                             clearColor = color::Fg(color::Reset)
                    );
                } else if status.as_u16() == 404 {
                    println!("{colorA}[unFollowing You] {colorB}{bold}{somebody}{clearColor}{clearStyle} (https://github.com/{somebody}) [{code}]",
                             somebody = somebody,
                             colorA = color::Fg(color::Red),
                             colorB = color::Fg(color::Blue),
                             bold = style::Bold,
                             clearStyle = style::Reset,
                             clearColor = color::Fg(color::Reset),
                             code = status.as_u16()
                    );
                } else {
                    error_count += 1;
                }
            },
            Err(_) => {
                error_count += 1;
            }
        }
    }

    println!("\nStatics: {} following you, {} not following you, {} error, {}% mutual following rate",
             following, total_followers - error_count - following, error_count, 100.0 * (following as f32) / ((total_followers - error_count) as f32));
    Ok(())
}
