#![allow(clippy::wildcard_imports)]
use anyhow::{Context, Result};
use reqwest::header::{self, HeaderMap};
use seed::{prelude::*, *};
use serde::Deserialize;

// TODO: 諸々、利用する各Github APIのレスポンスを確認してデータ構造の改善をする
#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    reviewers: Vec<Reviewer>, // TODO: レスポンスにない値なのでDeserialize失敗してるかも。要確認
}

#[derive(Debug, Deserialize)]
struct Reviewer {
    name: String,
    assigned_pull_requests: Vec<String>,
}

#[derive(Debug)]
struct Organization {
    name: String,
    repositories: Vec<Repository>,
}

struct Model {
    organization: Option<Organization>,
    error_message: Option<String>,
}

enum Msg {
    FetchData,
    DataFetched(Result<Organization>),
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        organization: None,
        error_message: None,
    }
}

#[wasm_bindgen(start)]
pub async fn start() {
    App::start("app", init, update, view);
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData => {
            model.organization = None;
            model.error_message = None;
            // TODO: perform_cmdはasync fnを引数に取れるのでコメントしたような渡し方は不要のはず。確認して削除する
            // let future = async { fetch_organization_data().map(Msg::DataFetched).await };
            let future = fetch_organization_data().map(Msg::DataFetched);
            orders.perform_cmd(future);
        }
        Msg::DataFetched(result) => match result {
            Ok(organization) => model.organization = Some(organization),
            Err(err) => model.error_message = Some(err.to_string()),
        },
    }
}

async fn fetch_organization_data() -> Result<Organization> {
    let organization_name = "my-organization";
    let access_token = "my-access-token";
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );
    headers.insert(header::USER_AGENT, "my-app".parse().unwrap());
    // セッションを再利用して複数回リクエストするためのインスタンスを生成する
    let client = reqwest::Client::new();
    let repositories_url = format!("https://api.github.com/orgs/{}/repos", organization_name);
    let repositories_response = &client
        .get(&repositories_url)
        .headers(headers.clone())
        .send()
        .await
        .with_context(|| format!("Failed to fetch repositories from {}", repositories_url))?
        .text()
        .await
        .with_context(|| "Failed to parse repositories response")?;
    let mut repositories: Vec<Repository> =
        serde_json::from_str(&repositories_response).unwrap_or_else(|_| Vec::new());
    for repository in &mut repositories {
        let pulls_url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=open",
            organization_name, repository.name
        );
        let pulls_response = &client
            .get(&pulls_url)
            .headers(headers.clone())
            .send()
            .await
            .with_context(|| format!("Failed to fetch pull requests from {}", pulls_url))?
            .text()
            .await
            .with_context(|| "Failed to parse pull requests response")?;
        let pulls: Vec<serde_json::Value> = serde_json::from_str(&pulls_response)
            .with_context(|| "Failed to parse pull requests")?;
        for pull in pulls {
            // TODO: Assignee不要なら消す
            // let empty_vec = Vec::new();
            // let assignees = match pull["assignees"].as_array() {
            //     Some(assignees) => assignees,
            //     None => &empty_vec,
            // };
            // let assignee_logins: Vec<String> = assignees
            //     .iter()
            //     .map(|a| a["login"].as_str().unwrap().to_string())
            //     .collect();
            let reviews_url = pull["url"]
                .as_str()
                .unwrap()
                .replace("api.", "")
                .replace("/pulls/", "/pulls/")
                + "/reviews";
            let reviews_response = &client
                .get(&reviews_url)
                .headers(headers.clone())
                .send()
                .await
                .with_context(|| format!("Failed to fetch reviews from {}", reviews_url))?
                .text()
                .await
                .with_context(|| "Failed to parse reviews response")?;
            let reviews: Vec<serde_json::Value> = serde_json::from_str(&reviews_response)
                .with_context(|| "Failed to parse reviews")?;
            for review in reviews {
                let reviewer_login = review["user"]["login"].as_str().unwrap().to_string();
                let state = review["state"].as_str().unwrap().to_string();
                if state != "COMMENTED" && state != "DISMISSED" {
                    if let Some(reviewer) = repository
                        .reviewers
                        .iter_mut()
                        .find(|r| r.name == reviewer_login)
                    {
                        reviewer
                            .assigned_pull_requests
                            .push(pull["url"].as_str().unwrap().to_string());
                    } else {
                        repository.reviewers.push(Reviewer {
                            name: reviewer_login.clone(),
                            assigned_pull_requests: vec![pull["url"].as_str().unwrap().to_string()],
                        });
                    }
                }
            }
        }
    }

    Ok(Organization {
        name: organization_name.to_string(),
        repositories,
    })
}

fn view(model: &Model) -> Node<Msg> {
    div![
        h1!("GitHub Organization Reviewers"),
        button!["Fetch data", ev(Ev::Click, |_| Msg::FetchData),],
        match &model.organization {
            Some(organization) => {
                div![
                    p![format!("Organization: {}", organization.name)],
                    div![
                        C!["repositories"],
                        organization.repositories.iter().map(|repository| {
                            div![
                                h2![&repository.name],
                                div![
                                    C!["reviewers"],
                                    repository.reviewers.iter().map(|reviewer| {
                                        div![
                                            C!["reviewer"],
                                            p![&reviewer.name],
                                            div![
                                                C!["pull-requests"],
                                                reviewer.assigned_pull_requests.iter().map(|url| {
                                                    a![
                                                        attrs! {
                                                        At::Href => url,
                                                        },
                                                        &url
                                                    ]
                                                }),
                                            ],
                                        ]
                                    }),
                                ],
                            ]
                        }),
                    ],
                ]
            }
            None => {
                match &model.error_message {
                    Some(error_message) => p![error_message],
                    None => p!["Click the button to fetch data."],
                }
            }
        }
    ]
}
