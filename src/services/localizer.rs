use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
    path::PathBuf,
};

use anyhow::Context;
use fluent_bundle::{concurrent::FluentBundle, FluentResource};
use serde::Deserialize;
use unic_langid::LanguageIdentifier;
use walkdir::WalkDir;

#[derive(Clone, Debug, Deserialize)]
pub struct I18nConfig {
    pub path: PathBuf,
}

pub struct Localizer {
    #[allow(unused)]
    resource_path: PathBuf,
    #[allow(unused)]
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
}

impl Localizer {
    pub fn new(config: I18nConfig) -> anyhow::Result<Self> {
        let parsed_entries = read_dir(&config.path)
            .and_then(|dir_items| dir_items.into_iter().collect::<Result<Vec<_>, _>>())
            .context("read i18n directory")?
            .into_iter()
            .filter(|dir_entry| dir_entry.file_type().is_ok_and(|ft| ft.is_dir()))
            .filter_map(|dir_entry| {
                dir_entry
                    .file_name()
                    .to_str()
                    .and_then(|dir_name| dir_name.parse::<LanguageIdentifier>().ok())
                    .map(|lang| (lang, dir_entry))
            })
            .collect::<Vec<_>>();
        let bundles = parsed_entries
            .into_iter()
            .map(|(lang, dir_entry)| {
                bundle_from(lang.clone(), dir_entry.path()).map(|bundle| (lang, bundle))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .context("create bundles")?;
        Ok(Self {
            resource_path: config.path,
            bundles,
        })
    }
}

fn bundle_from(
    lang: LanguageIdentifier,
    path: PathBuf,
) -> anyhow::Result<FluentBundle<FluentResource>> {
    let resources = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|e| e == "ftl"))
        .filter_map(|e| read_to_string(e.path()).ok())
        .map(FluentResource::try_new)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|(_, errors)| errors.into_iter().next().expect("errors to not be empty"))
        .context("parse fluent resource")?;
    let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);
    for resource in resources {
        bundle
            .add_resource(resource)
            .map_err(|errors| errors.into_iter().next().expect("errors to not be empty"))
            .context("add fluent resource")?;
    }
    Ok(bundle)
}
