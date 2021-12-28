use handlebars::Handlebars;
use std::{collections::HashSet, fs, path::Path};

use crate::model::{HomeView, IndexView, Recipe, RecipeView, SearchView, SiteMapView, SiteView};
use crate::template::{EscapeHelper, FNVHelper, LocaleHelper};

pub fn build_site(
    recipe_dir: &Path,
    static_dir: &Path,
    templates_dir: &Path,
    public_dir: &Path,
    site_locales: &[String],
    site: SiteView,
) -> Result<(), anyhow::Error> {
    let public_dir_exists: bool = Path::new(public_dir).is_dir();

    if public_dir_exists {
        fs::remove_dir_all(public_dir).expect("cannot remove output directory");
    }
    fs::create_dir_all(public_dir).expect("cannot create output directory");

    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    fs_extra::dir::copy(static_dir, public_dir, &options)?;

    let mut handlebars = Handlebars::new();

    handlebars.set_strict_mode(true);
    handlebars.register_helper("escape", Box::new(EscapeHelper));
    handlebars.register_helper("locale-helper", Box::new(LocaleHelper));
    handlebars.register_helper("fnv", Box::new(FNVHelper));
    handlebars
        .register_templates_directory(".hbs", templates_dir)
        .expect("cannot load templates");

    let recipe_files: Vec<String> = walkdir::WalkDir::new(recipe_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().display().to_string().ends_with(".yml"))
        .map(|e| e.path().display().to_string())
        .collect();

    let mut recipes: Vec<Recipe> = Vec::with_capacity(recipe_files.len());
    let mut recipe_ids: HashSet<String> = HashSet::new();

    let mut sitemap_paths: Vec<String> = vec!["about".to_string()];

    for recipe_file in &recipe_files {
        let recipe_yaml = fs::read_to_string(&recipe_file)?;
        let deserialized_recipe: Recipe = serde_yaml::from_str(&recipe_yaml)?;

        if !recipe_ids.insert(deserialized_recipe.id.to_string()) {
            error!("duplicate recipe id: {}", deserialized_recipe.id);
            continue;
        }

        recipes.push(deserialized_recipe);
    }

    for site_locale in site_locales {
        let locale_root = Path::new(public_dir).join(&site_locale);

        let mut search_views: Vec<SearchView> = Vec::with_capacity(recipe_files.len());

        sitemap_paths.push(site_locale.to_string());

        for recipe in &recipes {
            debug!("{}", recipe);

            for locale in &recipe.locales {
                if locale != site_locale {
                    continue;
                }

                let slug_root = &recipe.slug.clone().localized(Some(locale.clone()))?;

                let recipe_root = Path::new(&locale_root).join(slug_root);
                fs::create_dir_all(&recipe_root).unwrap_or_else(|_| {
                    panic!("unable to create recipe root {}", recipe_root.display())
                });

                let localized_recipe = recipe.to_partial(Some(locale.clone()), site_locales)?;

                sitemap_paths.push(format!("{}/{}", site_locale, slug_root));

                let recipe_html = handlebars
                    .render(
                        "recipe",
                        &RecipeView {
                            locale: site_locale.clone(),
                            title: format!("Just Recipes - {}", localized_recipe.name).to_string(),
                            recipe: localized_recipe.clone(),
                            site: site.clone(),
                            flat_steps: localized_recipe.flat_steps(),
                        },
                    )
                    .unwrap();
                // .unwrap_or_else(|_| panic!("unable to render recipe {}", recipe.id));
                let destination_html = recipe_root.join("index.html");
                fs::write(&destination_html, recipe_html).unwrap_or_else(|_| {
                    panic!(
                        "unable to write recipe html {} to {}",
                        recipe.id,
                        destination_html.display()
                    )
                });

                let recipe_json = serde_json::to_string(&recipe)?;
                let destination_json = recipe_root.join("index.json");
                fs::write(&destination_json, recipe_json).unwrap_or_else(|_| {
                    panic!(
                        "unable to write recipe json {} to {}",
                        recipe.id,
                        destination_json.display()
                    )
                });

                search_views.push(SearchView {
                    name: recipe.name.clone().localized(Some(locale.clone()))?,
                    link: format!(
                        "/{}/{}",
                        site_locale,
                        recipe.slug.clone().localized(Some(site_locale.clone()))?
                    ),
                });
            }
        }

        let index_html = handlebars
            .render(
                "recipe_list",
                &IndexView {
                    locale: site_locale.clone(),
                    title: "Just Recipes - Home".to_string(),
                    recipes: recipes.clone(),
                    site: site.clone(),
                },
            )
            .expect("unable to render index");
        let index_destination = Path::new(&locale_root).join("index.html");
        fs::write(&index_destination, index_html)
            .unwrap_or_else(|_| panic!("unable to write index to {}", index_destination.display()));

        let search_json = serde_json::to_string(&search_views)?;

        let search_destination = Path::new(&locale_root).join("search.json");
        fs::write(&search_destination, search_json).unwrap_or_else(|_| {
            panic!(
                "unable to write search json to {}",
                search_destination.display()
            )
        });
    }

    let home_html = handlebars
        .render(
            "index",
            &HomeView {
                locales: site_locales.to_vec(),
                title: "Just Recipes - Home".to_string(),
                site: site.clone(),
            },
        )
        .expect("unable to render index");
    let home_destination = Path::new(public_dir).join("index.html");
    fs::write(&home_destination, home_html)
        .unwrap_or_else(|_| panic!("unable to write home to {}", home_destination.display()));

    let about_html = handlebars
        .render(
            "about",
            &HomeView {
                locales: site_locales.to_vec(),
                title: "Just Recipes Blog - About".to_string(),
                site: site.clone(),
            },
        )
        .expect("unable to render about");
    let about_dir = Path::new(public_dir).join("about");
    fs::create_dir_all(&about_dir).expect("cannot create about directory");
    let about_destination = Path::new(&about_dir).join("index.html");
    fs::write(&about_destination, about_html)
        .unwrap_or_else(|_| panic!("unable to write about to {}", about_destination.display()));

    let sitemap_xml = handlebars
        .render(
            "sitemap",
            &SiteMapView {
                paths: sitemap_paths,
                site,
            },
        )
        .expect("unable to render sitemap");
    let sitemap_destination = Path::new(&public_dir).join("sitemap.xml");

    fs::write(&sitemap_destination, sitemap_xml).unwrap_or_else(|_| {
        panic!(
            "unable to write sitemap to {}",
            sitemap_destination.display()
        )
    });

    Ok(())
}
