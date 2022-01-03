use anyhow::{anyhow, Result};
use std::{collections::HashSet, fs, path::Path};
use uuid::Uuid;

use crate::model::{LocalizedString, Recipe, US_ENGLISH};

#[cfg(feature = "validate")]
pub fn validate_recipes(recipe_dir: &Path) -> Result<(), anyhow::Error> {
    let recipe_files: Vec<String> = walkdir::WalkDir::new(recipe_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().display().to_string().ends_with(".yml")
                || e.path().display().to_string().ends_with(".yaml")
        })
        .map(|e| e.path().display().to_string())
        .collect();

    let mut found_recipe_ids: HashSet<Uuid> = HashSet::new();
    let mut found_recipe_slugs: HashSet<String> = HashSet::new();

    for recipe_file in &recipe_files {
        let recipe_yaml = fs::read_to_string(&recipe_file)?;
        let (recipe_id, recipe_slugs) = validate_recipe(recipe_file, &recipe_yaml)?;

        if found_recipe_ids.contains(&recipe_id) {
            return Err(anyhow!("duplicate id {} in {}", recipe_id, recipe_file));
        }
        found_recipe_ids.insert(recipe_id);

        for recipe_slug in recipe_slugs {
            if found_recipe_slugs.contains(&recipe_slug) {
                return Err(anyhow!("duplicate slug {} in {}", recipe_slug, recipe_file));
            }
            found_recipe_slugs.insert(recipe_slug);
        }

        println!("OK: {}", recipe_file);
    }
    Ok(())
}

#[cfg(feature = "validate")]
pub fn validate_recipe(recipe_file_name: &str, recipe_yaml: &str) -> Result<(Uuid, Vec<String>)> {
    let deserialized_recipe: Recipe = serde_yaml::from_str(recipe_yaml)?;

    if deserialized_recipe.locales.is_empty() {
        return Err(anyhow!("locales cannot be empty"));
    }
    if deserialized_recipe.ingredients.is_empty() {
        return Err(anyhow!("ingredients cannot be empty"));
    }

    let categories = vec!["breakfast", "lunch", "beverage", "cocktail", "appetizer", "soup", "salad", "main dish", "side dish", "dessert", "break", "holiday", "entertaining"];

    validate_localized_string("name", &deserialized_recipe.name)?;
    validate_localized_string("slug", &deserialized_recipe.slug)?;
    validate_optional_localized_string("description", deserialized_recipe.description)?;
    validate_localized_strings("ingredients", deserialized_recipe.ingredients)?;
    validate_optional_localized_strings("equipment", deserialized_recipe.equipment)?;

    if deserialized_recipe.stages.is_empty() {
        return Err(anyhow!("stages cannot be empty"));
    }

    if deserialized_recipe.stages.len() > 5 {
        println!("WARNING: {} has more than 5 stages", recipe_file_name);
    }

    let mut step_count = 0;

    for stage in deserialized_recipe.stages {
        if stage.steps.is_empty() {
            return Err(anyhow!("steps cannot be empty"));
        }
        step_count += stage.steps.len();

        validate_localized_string("stage.name", &stage.name)?;
        validate_optional_localized_string("stage.description", stage.description)?;
        validate_optional_localized_string("stage.footer", stage.footer)?;
        validate_localized_strings("stage.steps", stage.steps)?;
    }

    if step_count > 20 {
        println!("WARNING: {} has over 20 steps", recipe_file_name);
    }

    Ok((deserialized_recipe.id, deserialized_recipe.slug.values()?))
}

#[cfg(feature = "validate")]
pub fn validate_localized_string(key: &str, value: &LocalizedString) -> Result<()> {
    if !value.inner.contains_key(US_ENGLISH) {
        return Err(anyhow!("{} must have en_US translation", key));
    }
    Ok(())
}

#[cfg(feature = "validate")]
pub fn validate_localized_strings(key: &str, values: Vec<LocalizedString>) -> Result<()> {
    for value in values {
        validate_localized_string(key, &value)?;
    }
    Ok(())
}

#[cfg(feature = "validate")]
pub fn validate_optional_localized_string(key: &str, value: Option<LocalizedString>) -> Result<()> {
    if value.is_some() {
        validate_localized_string(key, &value.unwrap())?;
    }
    Ok(())
}

#[cfg(feature = "validate")]
pub fn validate_optional_localized_strings(
    key: &str,
    values: Option<Vec<LocalizedString>>,
) -> Result<()> {
    if values.is_some() {
        for value in values.unwrap() {
            validate_localized_string(key, &value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! validate_recipe_parse_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                let res = validate_recipe("$name", input);
                assert!(res.is_err());
                assert_eq!(res.unwrap_err().to_string(), expected);
            }
        )*
        }
    }

    validate_recipe_parse_tests! {
            validate_recipe_err_missing_id: ("---", "invalid type: unit value, expected struct Recipe at line 2 column 1"),
            validate_recipe_err_missing_locales: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
", "missing field `locales` at line 2 column 3"),
            validate_recipe_err_missing_name: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
published: 2022-01-01
locales: []", "missing field `name` at line 2 column 3"),
            validate_recipe_err_missing_slug: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food", "missing field `slug` at line 2 column 3"),


            validate_recipe_err_missing_published: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food", "missing field `published` at line 2 column 3"),

            validate_recipe_err_missing_ingredients: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food", "missing field `ingredients` at line 2 column 3"),

            validate_recipe_err_missing_category: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
ingredients: []", "missing field `category` at line 2 column 3"),

            validate_recipe_err_missing_cuisine: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
category: dinner
ingredients: []", "missing field `cuisine` at line 2 column 3"),

    //         validate_recipe_err_missing_equipment: ("---
    // id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
    // locales: []
    // published: 2022-01-01
    // name: wonderful food
    // slug: 02e3f381de4e-wonderful-food
    // category: dinner
    // cuisine: american
    // ingredients: []", "missing field `equipment` at line 2 column 3"),

            validate_recipe_err_missing_stages: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
ingredients: []
category: dinner
cuisine: american
equipment: []", "missing field `stages` at line 2 column 3"),
            validate_recipe_err_empty_locales: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: []
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
category: dinner
cuisine: american
ingredients: []
equipment: []
stages: []", "locales cannot be empty"),
            validate_recipe_err_empty_ingredients: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food
ingredients: []
equipment: []
stages: []", "ingredients cannot be empty"),
            validate_recipe_err_empty_stages: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food
ingredients: [food_a]
equipment: []
stages: []", "stages cannot be empty"),
            validate_recipe_err_missing_steps: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food
ingredients: [food_a]
equipment: []
stages:
- name: prep", "stages[0]: missing field `steps` at line 12 column 7"),
            validate_recipe_err_empty_steps: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
category: dinner
cuisine: american
slug: 02e3f381de4e-wonderful-food
ingredients: [food_a]
equipment: []
stages:
- name: prep
  steps: []", "steps cannot be empty"),
            validate_recipe_err_name_locale: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name:
  en_GB: wonderful food
slug: 02e3f381de4e-wonderful-food
category: dinner
cuisine: american
ingredients: [food_a]
equipment: []
stages:
- name: prep
  steps:
  - first", "name must have en_US translation"),
            validate_recipe_err_stage_step_locale: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
category: dinner
cuisine: american
ingredients: [food_a]
equipment: []
stages:
- name: prep
  steps:
  - en_GB: first
", "stage.steps must have en_US translation"),
            validate_recipe_err_stage_invalid_duration: ("---
id: 56b7576b-efb2-4616-b2c4-02e3f381de4e
locales: [en_US]
published: 2022-01-01
name: wonderful food
slug: 02e3f381de4e-wonderful-food
category: dinner
cuisine: american
ingredients: [food_a]
equipment: []
stages:
- name: prep
  cook_time: invalid
  steps:
  - first
", "stages[0].cook_time: invalid value: string \"invalid\", expected a duration at line 13 column 14"),
        }
}
