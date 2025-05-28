use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use fluent_bundle::{concurrent::FluentBundle, FluentArgs, FluentResource, FluentValue};
use fluent_templates::{FluentLoader, Loader};
use serde::Deserialize;
use unic_langid::LanguageIdentifier;
use walkdir::WalkDir;

use crate::http::LocaleNegotiator;

use super::templating_engine::TemplateLocalizer;

#[derive(Clone, Debug, Deserialize)]
pub struct I18nConfig {
    pub path: PathBuf,
    pub fallback: LanguageIdentifier,
}

pub struct Localizer {
    path: PathBuf,
    fallback: LanguageIdentifier,
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
}

impl Localizer {
    pub fn new(config: I18nConfig) -> anyhow::Result<Self> {
        let I18nConfig { path, fallback } = config;
        let bundles = Self::read_bundles(&path)?;
        Ok(Self {
            path,
            fallback,
            bundles,
        })
    }

    fn read_bundles(
        resource_path: impl AsRef<Path>,
    ) -> anyhow::Result<HashMap<LanguageIdentifier, FluentBundle<FluentResource>>> {
        let parsed_entries = read_dir(resource_path)
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
        Ok(bundles)
    }

    fn lookup(
        &self,
        locale: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        let bundle = self.bundles.get(locale)?;
        let pattern = match text_id.split_once('.') {
            Some((msg, attr)) => bundle
                .get_message(msg)?
                .attributes()
                .find(|attribute| attribute.id() == attr)?
                .value(),
            None => bundle.get_message(text_id)?.value()?,
        };
        let args = args.map(|map| {
            map.iter()
                .map(|(k, v)| (k.as_ref(), v.clone()))
                .collect::<FluentArgs<'_>>()
        });
        let mut errors = Vec::new();
        let value = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
        errors.is_empty().then(|| value.into())
    }

    fn locales(&self) -> impl Iterator<Item = &LanguageIdentifier> {
        self.bundles.keys()
    }
}

impl LocaleNegotiator for Localizer {
    fn negotiate_locale(&self, sorted_preferences: Vec<LanguageIdentifier>) -> LanguageIdentifier {
        sorted_preferences
            .into_iter()
            .filter_map(|preference| {
                self.locales()
                    .find(|supported| supported.language == preference.language)
            })
            .next()
            .unwrap_or(&self.fallback)
            .clone()
    }
}

impl TemplateLocalizer for Arc<Localizer> {
    fn reload(&mut self) -> crate::Result<()> {
        let Localizer {
            path,
            fallback,
            bundles: _,
        } = self.as_ref();
        let bundles = Localizer::read_bundles(path)
            .context("reload localizer")?;
        let localizer = Localizer {
            path: path.clone(),
            fallback: fallback.clone(),
            bundles,
        };
        *self = Arc::new(localizer);
        Ok(())
    }

    fn into_function(self) -> impl tera::Function {
        FluentLoader::new(ArcLocalizer(self.clone()))
    }
}

struct ArcLocalizer(Arc<Localizer>);

impl Loader for ArcLocalizer {
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String {
        self.try_lookup_complete(lang, text_id, args)
            .unwrap_or_else(|| "translation error".to_owned())
    }

    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        self.0.lookup(lang, text_id, args)
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.0.locales())
    }
}

fn bundle_from(
    locale: LanguageIdentifier,
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
    let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
    for resource in resources {
        bundle
            .add_resource(resource)
            .map_err(|errors| errors.into_iter().next().expect("errors to not be empty"))
            .context("add fluent resource")?;
    }
    Ok(bundle)
}
