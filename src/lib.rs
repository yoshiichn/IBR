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

#[derive(Clone, Debug, Deserialize)]
struct Reviewer {
    name: String,
    assigned_pull_requests: Vec<PullRequest>,
}

#[derive(Clone, Debug, Deserialize)]
struct PullRequest {
    id: String,
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
    let pr1 = PullRequest {
        id: "id-1".to_string(),
        repo_name: "repo1".to_string(),
    };
    let pr2 = PullRequest {
        id: "id-2".to_string(),
        repo_name: "repo2".to_string(),
    };
    let pr3 = PullRequest {
        id: "id-3".to_string(),
        repo_name: "repo1".to_string(),
    };
    let reviewer1 = Reviewer {
        name: "user-1".to_string(),
        assigned_pull_requests: vec![pr1.clone(), pr2.clone(), pr3.clone()],
    };
    let reviewer2 = Reviewer {
        name: "user-2".to_string(),
        assigned_pull_requests: vec![pr1.clone(), pr2.clone()],
    };
    Ok(Organization {
        name: "my-org".to_string(),
        reviewers: vec![reviewer1, reviewer2],
        repositories: vec![Repository {name: "repo1".to_string()}, Repository {name: "repo2".to_string()}]
    })
}

fn view(model: &Model) -> Node<Msg> {
    div![
        h1![
            a![
                attrs! {
                    At::Href => "https://github.com/yoshiichn/IBR",
                    At::Target => "_blank",
                    At::Rel => "noopener noreferrer",
                },
                format!("{} I'm Busy Reviewing. {}", '\u{1F347}', '\u{1F980}')
            ]
        ],
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
                        tbody![
                            organization.reviewers.iter().map(|reviewer| {
                                tr![
                                    td![
                                        style![
                                            St::Padding => "10px",
                                            St::VerticalAlign => "top",
                                        ],
                                        &reviewer.name
                                    ],
                                    organization.repositories.iter().map(|repo| {
                                        let prs: Vec<PullRequest> = reviewer.assigned_pull_requests.iter().filter(|pr| pr.repo_name == *repo.name).cloned().collect();
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
                            })
                        ]
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
