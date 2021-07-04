use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    query: String,
    #[clap(long)]
    with_details: bool,
    #[clap(short, long, default_value = "20")]
    limit: u32,
    #[clap(short, long, default_value = "0")]
    offset: u32,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = search(&self, args.cfg).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn search(args: &Input, cfg: &DsConfig) -> Result<Response, CmdError> {
    let url = format!("{}/api/v1/sec/item/search", cfg.docspell_url);
    let client = reqwest::blocking::Client::new();
    let token = login::session_token()?;
    client
        .get(url)
        .header(login::DOCSPELL_AUTH, token)
        .query(&[
            ("limit", &args.limit.to_string()),
            ("offset", &args.offset.to_string()),
            ("withDetails", &args.with_details.to_string()),
            ("q", &args.query),
        ])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<Response>()
        .map_err(CmdError::HttpError)
}

#[derive(Debug, Serialize, Deserialize)]
struct IdName {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Attach {
    id: String,
    position: u32,
    name: String,
    pageCount: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tag {
    id: String,
    name: String,
    category: Option<String>,
    created: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomField {
    id: String,
    name: String,
    label: Option<String>,
    ftype: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Highlight {
    name: String,
    lines: Box<[String]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    state: String,
    date: u64,
    dueDate: Option<u64>,
    source: String,
    direction: Option<String>,
    #[serde(alias = "corrOrg")]
    corr_org: Option<IdName>,
    #[serde(alias = "corrPerson")]
    corr_person: Option<IdName>,
    #[serde(alias = "concPerson")]
    conc_person: Option<IdName>,
    #[serde(alias = "concEquipment")]
    conc_equip: Option<IdName>,
    folder: Option<IdName>,
    attachments: Box<[Attach]>,
    tags: Box<[Tag]>,
    customfields: Box<[CustomField]>,
    notes: Option<String>,
    highlighting: Box<[Highlight]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Group {
    name: String,
    items: Box<[Item]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    groups: Box<[Group]>,
}
