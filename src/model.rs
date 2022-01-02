use anyhow::anyhow;
use humantime::format_duration;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use serde::{Deserialize as DeserializeMacro, Serialize as SerializeMacro};
use slugify::slugify;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

use crate::when::duration_iso8601;

pub const US_ENGLISH: &str = "en_US";

#[derive(Debug, Clone, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct Recipe {
    pub id: Uuid,
    pub locales: Vec<String>,
    pub name: LocalizedString,
    pub slug: LocalizedString,
    pub description: Option<LocalizedString>,
    pub ingredients: Vec<LocalizedString>,
    pub equipment: Vec<LocalizedString>,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct Stage {
    pub name: LocalizedString,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub cook_time: Option<Duration>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub prep_time: Option<Duration>,

    pub description: Option<LocalizedString>,
    pub footer: Option<LocalizedString>,
    pub steps: Vec<LocalizedString>,
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Recipe {} ({})", self.name, self.id)
    }
}

impl Recipe {
    pub fn to_partial(
        &self,
        locale: Option<String>,
        allowed_locales: &[String],
        images: Vec<(String, String)>,
    ) -> Result<RecipePartial, anyhow::Error> {
        let cook_time: Duration = self
            .stages
            .clone()
            .into_iter()
            .fold(Duration::new(0, 0), |sum, val| {
                sum + val.cook_time.unwrap_or_default()
            });
        let prep_time: Duration = self
            .stages
            .clone()
            .into_iter()
            .fold(Duration::new(0, 0), |sum, val| {
                sum + val.prep_time.unwrap_or_default()
            });
        let total_time = cook_time + prep_time;

        Ok(RecipePartial {
            id: self.id,
            alternate_locales: self
                .locales
                .clone()
                .into_iter()
                .take_while(|e| allowed_locales.contains(e))
                .map(|l| (l.clone(), self.slug.clone().localized(Some(l)).unwrap()))
                .collect(),
            cook_time: match cook_time.is_zero() {
                false => Some(format_duration(cook_time).to_string()),
                true => None,
            },
            prep_time: match prep_time.is_zero() {
                false => Some(format_duration(prep_time).to_string()),
                true => None,
            },
            total_time: match total_time.is_zero() {
                false => Some(format_duration(total_time).to_string()),
                true => None,
            },
            sd_cook_time: match cook_time.is_zero() {
                false => Some(duration_iso8601(cook_time)),
                true => None,
            },
            sd_prep_time: match prep_time.is_zero() {
                false => Some(duration_iso8601(prep_time)),
                true => None,
            },
            name: self.name.clone().localized(locale.clone())?,
            slug: self.slug.clone().localized(locale.clone())?,
            description: match self.description.clone() {
                Some(x) => Some(x.localized(locale.clone())?),
                None => None,
            },
            ingredients: localied_vec(&self.ingredients, locale.clone())?,
            equipment: localied_vec(&self.equipment, locale.clone())?,
            stages: self
                .stages
                .clone()
                .into_iter()
                .map(|x| x.to_partial(locale.clone()).unwrap())
                .collect(),
            images,
        })
    }

    pub fn init(arg_recipe_id: Option<Uuid>, arg_name: Option<String>, mock: bool) -> Self {
        let recipe_id: Uuid = match arg_recipe_id {
            Some(value) => value,
            None => Uuid::new_v4(),
        };
        let name: String = match arg_name {
            Some(value) => value,
            None => "A wonderful new recipe".to_string(),
        };
        let short_id: String = recipe_id
            .to_hyphenated()
            .to_string()
            .chars()
            .rev()
            .into_iter()
            .take(12)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let slug: String = slugify!(format!("{}-{}", short_id, name).as_str());
        let description: Option<LocalizedString> = match mock {
            true => Some(LocalizedString::new(
                &"This recipe is pretty neat.".to_string(),
            )),
            false => None,
        };
        let ingredients: Vec<LocalizedString> = match mock {
            true => vec![
                LocalizedString::new(&"celery".to_string()),
                LocalizedString::new(&"onion".to_string()),
                LocalizedString::new(&"bell pepper".to_string()),
            ],
            false => Vec::new(),
        };
        let equipment: Vec<LocalizedString> = match mock {
            true => vec![LocalizedString::new(&"dutch oven".to_string())],
            false => Vec::new(),
        };
        let stages: Vec<Stage> = match mock {
            true => vec![Stage::init("Cook".to_string())],
            false => Vec::new(),
        };
        Recipe {
            id: recipe_id,
            locales: vec![US_ENGLISH.to_string()],
            name: LocalizedString::new(&name),
            slug: LocalizedString::new(&slug),
            description,

            ingredients,
            equipment,
            stages,
        }
    }
}

impl Stage {
    pub fn to_partial(&self, locale: Option<String>) -> Result<StagePartial, anyhow::Error> {
        let cook_time = self.cook_time.unwrap_or_default();
        let prep_time = self.prep_time.unwrap_or_default();
        let total_time = cook_time + prep_time;

        Ok(StagePartial {
            name: self.name.clone().localized(locale.clone())?,
            cook_time: match cook_time.is_zero() {
                false => Some(format_duration(cook_time).to_string()),
                true => None,
            },
            prep_time: match prep_time.is_zero() {
                false => Some(format_duration(prep_time).to_string()),
                true => None,
            },
            total_time: match total_time.is_zero() {
                false => Some(format_duration(total_time).to_string()),
                true => None,
            },
            description: match self.description.clone() {
                Some(x) => Some(x.localized(locale.clone())?),
                None => None,
            },
            footer: match self.footer.clone() {
                Some(x) => Some(x.localized(locale.clone())?),
                None => None,
            },
            steps: localied_vec(&self.steps, locale)?,
        })
    }

    pub fn init(name: String) -> Self {
        Stage {
            name: LocalizedString::new(&name),
            cook_time: None,
            prep_time: None,
            description: None,
            footer: None,
            steps: vec![
                LocalizedString::new(&"First do this".to_string()),
                LocalizedString::new(&"Then do that".to_string()),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct RecipePartial {
    pub id: Uuid,
    pub alternate_locales: Vec<(String, String)>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub ingredients: Vec<String>,
    pub equipment: Vec<String>,
    pub stages: Vec<StagePartial>,
    pub cook_time: Option<String>,
    pub prep_time: Option<String>,
    pub total_time: Option<String>,
    pub sd_cook_time: Option<String>,
    pub sd_prep_time: Option<String>,
    pub images: Vec<(String, String)>,
}

impl RecipePartial {
    pub fn flat_steps(&self) -> Vec<String> {
        let size = (&self.stages).iter().map(|s| s.steps.len()).sum();
        let mut all_steps = Vec::with_capacity(size);
        for stage in &self.stages {
            all_steps.extend(stage.steps.clone());
        }
        all_steps
    }
}

#[derive(Debug, Clone, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct StagePartial {
    pub name: String,
    pub cook_time: Option<String>,
    pub prep_time: Option<String>,
    pub total_time: Option<String>,
    pub description: Option<String>,
    pub footer: Option<String>,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct SiteView {
    pub public_url: String,
    pub version: String,
}

impl SiteView {
    pub fn new(public_url: &str, version: &str) -> Self {
        SiteView {
            public_url: public_url.to_string(),
            version: version.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct HomeView {
    pub locales: Vec<String>,
    pub title: String,
    pub site: SiteView,
    pub self_url: String,
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct IndexView {
    pub locale: String,
    pub title: String,
    pub recipes: Vec<Recipe>,
    pub site: SiteView,
    pub self_url: String,
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct RecipeView {
    pub locale: String,
    pub title: String,
    pub recipe: RecipePartial,
    pub site: SiteView,
    pub flat_steps: Vec<String>,
    pub self_url: String,
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct PageView {
    pub locale: String,
    pub title: String,
    pub content: String,
    pub site: SiteView,
    pub self_url: String,
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct SearchView {
    pub name: String,
    pub link: String,
}

#[derive(Debug, PartialEq, SerializeMacro, DeserializeMacro)]
pub struct SiteMapView {
    pub paths: Vec<String>,
    pub site: SiteView,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalizedString {
    pub inner: HashMap<String, String>,
}

impl LocalizedString {
    pub fn localized(&self, locale: Option<String>) -> Result<String, anyhow::Error> {
        let search_locale = locale.unwrap_or_else(|| US_ENGLISH.to_string());
        if let Some(value) = self.inner.get(&search_locale) {
            return Ok(value.to_string());
        }
        if let Some(value) = self.inner.get(&US_ENGLISH.to_string()) {
            return Ok(value.to_string());
        }
        return Err(anyhow!("Missing locale: {}", search_locale));
    }

    pub fn new(value: &str) -> Self {
        let inner = HashMap::from([(US_ENGLISH.to_string(), value.to_string())]);
        LocalizedString { inner }
    }

    pub fn values(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(self.inner.values().cloned().collect::<Vec<String>>())
    }
}

impl fmt::Display for LocalizedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.inner
                .clone()
                .into_iter()
                .into_iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl Serialize for LocalizedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(self.inner.iter())
    }
}

impl<'de> Deserialize<'de> for LocalizedString {
    fn deserialize<D>(deserializer: D) -> Result<LocalizedString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LocalizedStringVisitor)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalizedStringVisitor;

impl<'de> Visitor<'de> for LocalizedStringVisitor {
    type Value = LocalizedString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("en_US string or map")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut out: HashMap<String, String> = HashMap::new();
        out.insert(US_ENGLISH.to_string(), v.to_string());
        Ok(LocalizedString { inner: out })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut out: HashMap<String, String> = HashMap::new();

        while let Some((key, value)) = map.next_entry::<String, String>()? {
            out.insert(key.to_string(), value.to_string());
        }

        Ok(LocalizedString { inner: out })
    }
}

pub fn localied_vec(
    values: &[LocalizedString],
    locale: Option<String>,
) -> Result<Vec<String>, anyhow::Error> {
    let mut results: Vec<String> = Vec::with_capacity(values.len());

    for (index, value) in values.iter().enumerate() {
        results.insert(index, value.clone().localized(locale.clone())?.clone());
    }

    Ok(results)
}
