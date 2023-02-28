#![allow(clippy::wildcard_imports)]
use anyhow::{Context, Result};
use reqwest::header::{self, HeaderMap};
use seed::{prelude::*, *};
use serde::Deserialize;

#[derive(Debug)]
struct Organization {
    name: String,
    reviewers: Vec<Reviewer>,
    repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Reviewer {
    name: String,
    assigned_pull_requests: Vec<PullRequest>,
}

#[derive(Debug, Deserialize, Clone)]
struct PullRequest {
    id: String,
    url: String,
    repo_name: String,
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
    let organization_name = "your-organization";
    let access_token = "your-access-token";
    let mut org = Organization {
        name: organization_name.to_string(),
        reviewers: vec![],
        repositories: vec![],
    };
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
    org.repositories = serde_json::from_str(&repositories_response).unwrap_or_else(|_| Vec::new());
    for repository in &mut org.repositories {
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
            // pullにデータがあるものだけ動かす
            org.reviewers.push(Reviewer {
                name: pull["requested_reviewers"][0]["login"].to_string(),
                assigned_pull_requests: vec![],
            });
            org.reviewers[0].assigned_pull_requests.push(PullRequest {
                id: pull["number"].to_string(),
                url: pull["url"]
                    .as_str()
                    .unwrap()
                    .replace("api.", "")
                    .replace("repos/", "")
                    .replace("pulls", "pull"),
                repo_name: repository.name.to_string(),
            });
            // let reviews_response = &client
            //     .get(&reviews_url)
            //     .headers(headers.clone())
            //     .send()
            //     .await
            //     .with_context(|| format!("Failed to fetch reviews from {}", reviews_url))?
            //     .text()
            //     .await
            //     .with_context(|| "Failed to parse reviews response")?;
            // let reviews: Vec<serde_json::Value> = serde_json::from_str(&reviews_response)
            //     .with_context(|| "Failed to parse reviews")?;
            // for review in reviews {
            //     let reviewer_login = review["user"]["login"].as_str().unwrap().to_string();
            //     let state = review["state"].as_str().unwrap().to_string();
            // }
        }
    }

    Ok(org)
}

fn view(model: &Model) -> Node<Msg> {
    div![
        h1![a![
            attrs! {
                At::Href => "https://github.com/yoshiichn/IBR",
                At::Target => "_blank",
                At::Rel => "noopener noreferrer",
            },
            format!("{} I'm Busy Reviewing. {}", '\u{1F347}', '\u{1F980}')
        ]],
        button![
            "Fetch data",
            ev(Ev::Click, |_| Msg::FetchData),
            style![
                St::BackgroundColor => "#2c3e50",
                St::Color => "#ffffff",
                St::Padding => "10px 20px",
                St::BorderRadius => "5px",
                St::Cursor => "pointer",
            ],
        ],
        match &model.organization {
            Some(organization) => {
                div![
                    p![
                        style![
                            St::FontWeight => "bold",
                            St::FontSize => "18px",
                            St::MarginBottom => "10px",
                        ],
                        format!("Organization: {}", organization.name)
                    ],
                    table![
                        style![
                            St::BorderCollapse => "collapse",
                            St::Width => "100%",
                            St::MarginBottom => "20px",
                        ],
                        thead![
                            style![
                                St::BackgroundColor => "#2c3e50",
                                St::Color => "#ffffff",
                            ],
                            tr![
                                style![
                                    St::FontWeight => "bold",
                                    St::Padding => "10px",
                                    St::TextAlign => "left",
                                ],
                                th!["Users"],
                                organization.repositories.iter().map(|repo| {
                                    th![
                                        style![
                                            St::Padding => "10px",
                                            St::TextAlign => "center",
                                        ],
                                        &repo.name
                                    ]
                                })
                            ]
                        ],
                        tbody![organization.reviewers.iter().map(|reviewer| {
                            tr![
                                td![
                                    style![
                                        St::Padding => "10px",
                                        St::VerticalAlign => "top",
                                    ],
                                    &reviewer.name
                                ],
                                organization.repositories.iter().map(|repo| {
                                    let prs: Vec<PullRequest> = reviewer
                                        .assigned_pull_requests
                                        .iter()
                                        .filter(|pr| pr.repo_name == *repo.name)
                                        .cloned()
                                        .collect();
                                    td![
                                        style![
                                            St::Padding => "10px",
                                            St::VerticalAlign => "top",
                                            St::TextAlign => "center",
                                        ],
                                        prs.iter().map(|pr| {
                                            a![
                                                style![
                                                    St::BackgroundColor => "#3498db",
                                                    St::Color => "#ffffff",
                                                    St::TextDecoration => "none",
                                                    St::Padding => "5px 10px",
                                                    St::BorderRadius => "5px",
                                                    St::Cursor => "pointer",
                                                ],
                                                &pr.id
                                            ]
                                        })
                                    ]
                                })
                            ]
                        })]
                    ]
                ]
            }
            None => match &model.error_message {
                Some(error_message) => p![
                    style![
                        St::FontWeight => "bold",
                        St::Color => "red",
                    ],
                    error_message
                ],
                None => p![
                    style![
                        St::FontWeight => "bold",
                    ],
                    "Click the button to fetch data."
                ],
            },
        }
    ]
}
