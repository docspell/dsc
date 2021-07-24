mod common;
use crate::common::{mk_cmd, Result};
use assert_cmd::prelude::*;
use dsc::http::payload::{BasicResult, SearchResult, SourceAndTags, Summary};
use std::process::Command;

#[test]
fn dsc_help() -> Result<()> {
    let mut cmd = Command::cargo_bin("dsc")?;
    let assert = cmd.arg("--help").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn remote_version() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.arg("version").assert();
    assert.success().stderr("");
    Ok(())
}

#[test]
fn remote_register() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("register")
        .args(&["-c", "demo2", "-l", "demo2", "--password", "test"])
        .output()?;

    let out: BasicResult = serde_json::from_slice(out.stdout.as_slice())?;
    assert!(out.success);
    Ok(())
}

#[test]
fn remote_upload_web() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.arg("upload").arg("README.md").assert();
    assert
        .success()
        .stdout("{\"success\":true,\"message\":\"Files submitted.\"}");
    Ok(())
}

#[test]
fn remote_upload_int_endpoint() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd
        .arg("upload")
        .args(&[
            "-c",
            "demo",
            "-i",
            "--header",
            "Docspell-Integration:test123",
        ])
        .arg("README.md")
        .assert();
    assert
        .success()
        .stdout("{\"success\":true,\"message\":\"Files submitted.\"}");
    Ok(())
}

#[test]
fn remote_source_list() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd.arg("source").arg("list").output()?;

    let out: Vec<SourceAndTags> = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].source.id,
        "FcVZWHAgfFD-MdYCm3qWTyX-a7hcbVhsgKG-FG9DwArw9eQ"
    );
    Ok(())
}

#[test]
fn remote_source_list_filter_id() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("source")
        .arg("list")
        .arg("--id")
        .arg("FcVZ")
        .output()?;

    let out: Vec<SourceAndTags> = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].source.id,
        "FcVZWHAgfFD-MdYCm3qWTyX-a7hcbVhsgKG-FG9DwArw9eQ"
    );
    Ok(())
}

#[test]
fn remote_source_list_filter_id_neg() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("source")
        .arg("list")
        .arg("--id")
        .arg("xyz")
        .output()?;

    let out: Vec<SourceAndTags> = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.len(), 0);
    Ok(())
}

#[test]
fn remote_source_list_filter_name() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("source")
        .arg("list")
        .arg("--name")
        .arg("test")
        .output()?;

    let out: Vec<SourceAndTags> = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].source.id,
        "FcVZWHAgfFD-MdYCm3qWTyX-a7hcbVhsgKG-FG9DwArw9eQ"
    );
    Ok(())
}

#[test]
fn remote_source_list_filter_name_neg() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("source")
        .arg("list")
        .arg("--name")
        .arg("xyz")
        .output()?;

    let out: Vec<SourceAndTags> = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.len(), 0);
    Ok(())
}

#[test]
fn remote_search_1() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd.arg("search").arg("name:*").output()?;

    let out: SearchResult = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(out.groups.len(), 2);
    assert_eq!(out.groups[0].name, "2019-09");
    assert_eq!(out.groups[1].name, "2016-01");
    Ok(())
}

#[test]
fn remote_search_2() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd.arg("search").arg("corr:pancake*").output()?;

    let res: SearchResult = serde_json::from_slice(out.stdout.as_slice())?;
    assert_eq!(res.groups.len(), 1);
    assert_eq!(res.groups[0].name, "2019-09");
    Ok(())
}

#[test]
fn remote_upload_source() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd
        .arg("upload")
        .arg("--source")
        .arg("FcVZWHAgfFD-MdYCm3qWTyX-a7hcbVhsgKG-FG9DwArw9eQ")
        .arg("README.md")
        .assert();
    assert.success();
    Ok(())
}

#[test]
fn remote_search_summary() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd.arg("search-summary").arg("name:*").output()?;

    let res: Summary = serde_json::from_slice(out.stdout.as_slice())?;

    assert_eq!(res.count, 2);
    assert_eq!(res.tag_cloud.items.len(), 5);
    assert_eq!(res.tag_category_cloud.items.len(), 2);
    assert_eq!(res.field_stats.len(), 2);
    Ok(())
}

#[test]
fn remote_download() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("download")
        .arg("--target")
        .arg("files_test")
        .arg("date<today")
        .assert();

    out.success().stderr("");
    let files = std::fs::read_dir("files_test/").unwrap().count();
    assert_eq!(files, 2);

    std::fs::remove_dir_all("files_test/").unwrap();
    Ok(())
}

#[test]
fn remote_download_zip() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let out = cmd
        .arg("download")
        .arg("--target")
        .arg("zip_test/test.zip")
        .arg("--zip")
        .arg("date<today")
        .assert();

    out.success().stderr("");
    let zip = std::path::PathBuf::from("zip_test/test.zip");
    assert!(zip.exists());

    std::fs::remove_dir_all("zip_test/").unwrap();
    Ok(())
}

// #[test]
// fn remote_admin_convert_all_pdfs() -> Result<()> {
//     let mut cmd = mk_cmd()?;
//     let out = cmd.arg("admin").arg("convert-all-pdfs").assert();

//     out.success().stderr("").stdout("");
//     Ok(())
// }
