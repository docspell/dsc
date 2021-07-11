use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::{EndpointOpts, UploadMeta};
use crate::types::{BasicResult, StringList, UploadMeta as MetaRequest, DOCSPELL_AUTH};
use crate::{cmd::login, config::DsConfig};
use clap::{Clap, ValueHint};
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::{Client, RequestBuilder};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Uploads files to docspell.
///
///
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    #[clap(flatten)]
    pub upload: UploadMeta,

    /// Use the given source id. If not specified, the default id from
    /// the config is used or a login is required
    #[clap(long, group = "int")]
    pub source: Option<String>,

    /// One or more files to upload
    #[clap(required = true, min_values = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
}
impl Input {
    fn collective_id(&self) -> Result<&String, CmdError> {
        self.endpoint
            .collective
            .as_ref()
            .ok_or(CmdError::InvalidInput(
                "Collective must be present when using integration endpoint.".into(),
            ))
    }

    fn source_id(&self, cfg: &DsConfig) -> Option<String> {
        self.source.clone().or(cfg.default_source_id.clone())
    }
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = upload_files(self, args)?;
        args.write_result(result)?;

        Ok(())
    }
}

fn upload_files(args: &Input, cfg: &CmdArgs) -> Result<BasicResult, CmdError> {
    let url = if args.endpoint.integration {
        let coll_id = args.collective_id()?;
        format!(
            "{}/api/v1/open/integration/item/{}",
            cfg.cfg.docspell_url, coll_id
        )
    } else {
        match &args.source_id(cfg.cfg) {
            Some(id) => format!("{}/api/v1/open/upload/item/{}", cfg.docspell_url(), id),
            None => format!("{}/api/v1/sec/upload/item", cfg.docspell_url()),
        }
    };

    let meta = MetaRequest {
        multiple: args.upload.multiple,
        direction: args.upload.direction.clone(),
        folder: args.upload.folder.clone(),
        skip_duplicates: args.upload.skip_duplicates,
        tags: StringList {
            items: args.upload.tag.clone(),
        },
        file_filter: args.upload.file_filter.clone(),
        language: args.upload.language.clone(),
    };
    let meta_json = serde_json::to_vec(&meta)?;
    let meta_part = Part::bytes(meta_json).mime_str("application/json")?;
    log::debug!("Send file metadata: {}", serde_json::to_string(&meta)?);
    let mut form = Form::new().part("meta", meta_part);
    for path in &args.files {
        //TODO seems that async is the only way to use a byte stream??
        let mut fopen = File::open(path)?;
        let len = fopen.metadata()?.len();
        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
        fopen.read_to_end(&mut buffer)?;
        let mut fpart = Part::bytes(buffer);
        if let Some(fname) = path.as_path().file_name() {
            let f: String = fname.to_string_lossy().into();
            fpart = fpart.file_name(f);
        }
        form = form.part("file", fpart);
    }

    let client = create_client(&url, args, cfg)?;
    client
        .multipart(form)
        .send()
        .and_then(|r| r.error_for_status())?
        .json::<BasicResult>()
        .map_err(CmdError::HttpError)
}

fn create_client(url: &str, opts: &Input, args: &CmdArgs) -> Result<RequestBuilder, CmdError> {
    if opts.source_id(args.cfg).is_none() && !opts.endpoint.integration {
        let token = login::session_token(args)?;
        Ok(Client::new().post(url).header(DOCSPELL_AUTH, token))
    } else {
        let mut c = Client::new().post(url);
        c = opts.endpoint.apply(c);
        Ok(c)
    }
}
