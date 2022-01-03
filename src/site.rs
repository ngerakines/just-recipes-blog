use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use itertools::Itertools;
use slugify::slugify;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
use url::Url;

use crate::model::{HomeView, LinkListView, Recipe, RecipeView, SearchView, SiteMapView, SiteView};
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
    handlebars.register_helper(
        "url",
        Box::new(
            |h: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                let joined: String = h
                    .params()
                    .iter()
                    .map(|id| match id.value().as_str() {
                        Some(value) => format!("{}/", value),
                        None => "".to_string(),
                    })
                    .collect();
                let combined = format!("{}{}", site.public_url, joined);
                out.write(&combined)?;
                Ok(())
            },
        ),
    );

    handlebars.register_script_helper_file("incr", templates_dir.join("incr.rhai"))?;
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

    let mut site_links: HashSet<String> = HashSet::new();
    site_links.insert(site.public_url.clone());

    for recipe_file in &recipe_files {
        let recipe_yaml = fs::read_to_string(&recipe_file)?;
        let deserialized_recipe: Recipe = serde_yaml::from_str(&recipe_yaml)?;

        if !recipe_ids.insert(deserialized_recipe.id.to_string()) {
            error!("duplicate recipe id: {}", deserialized_recipe.id);
            continue;
        }

        recipes.push(deserialized_recipe);
    }

    recipes.sort_by(|a, b| {
        a.name
            .localized(None)
            .unwrap()
            .cmp(&b.name.localized(None).unwrap())
    });

    for site_locale in site_locales {
        let locale_root = Path::new(public_dir).join(&site_locale);

        let mut search_views: Vec<SearchView> = Vec::with_capacity(recipe_files.len());

        site_links.insert(format!("{}{}/", site.public_url, site_locale));

        let mut categorized_recipes: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut cuisine_recipes: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut recipe_links: Vec<(String, String)> = vec![];

        for recipe in &recipes {
            debug!("{}", recipe);

            let image_path = Path::new(&recipe_dir).join(format!("{}.jpg", recipe.id));
            let thumbnail_path =
                Path::new(&recipe_dir).join(format!("{}_thumbnail.jpg", recipe.id));

            let mut images: Vec<(String, String)> = Vec::new();
            let has_images = image_path.exists() && thumbnail_path.exists();
            if has_images {
                images.push((
                    format!("{}_thumbnail.jpg", recipe.id),
                    format!("{}.jpg", recipe.id),
                ));
            }

            for locale in &recipe.locales {
                if locale != site_locale {
                    continue;
                }

                let slug_root = &recipe.slug.clone().localized(Some(locale.clone()))?;

                let recipe_root = Path::new(&locale_root).join(slug_root);
                fs::create_dir_all(&recipe_root).unwrap_or_else(|_| {
                    panic!("unable to create recipe root {}", recipe_root.display())
                });

                if has_images {
                    fs::copy(&image_path, recipe_root.join(format!("{}.jpg", recipe.id)))?;
                    fs::copy(
                        &thumbnail_path,
                        recipe_root.join(format!("{}_thumbnail.jpg", recipe.id)),
                    )?;
                }

                let localized_recipe =
                    recipe.to_partial(Some(locale.clone()), site_locales, images.clone())?;

                let self_url = format!(
                    "{}{}/{}/",
                    site.public_url, site_locale, localized_recipe.slug
                );
                site_links.insert(self_url.clone());

                recipe_links.push((self_url.clone(), localized_recipe.name.clone()));

                if let Some(x) = categorized_recipes.get_mut(&localized_recipe.category) {
                    x.push((
                        String::from(&self_url),
                        String::from(&localized_recipe.name),
                    ));
                } else {
                    categorized_recipes.insert(
                        localized_recipe.category.clone(),
                        vec![(
                            String::from(&self_url),
                            String::from(&localized_recipe.name),
                        )],
                    );
                }

                if let Some(x) = cuisine_recipes.get_mut(&localized_recipe.cuisine) {
                    x.push((
                        String::from(&self_url),
                        String::from(&localized_recipe.name),
                    ));
                } else {
                    cuisine_recipes.insert(
                        localized_recipe.cuisine.clone(),
                        vec![(
                            String::from(&self_url),
                            String::from(&localized_recipe.name),
                        )],
                    );
                }

                let recipe_html = handlebars
                    .render(
                        "recipe",
                        &RecipeView {
                            locale: site_locale.clone(),
                            title: format!("Just Recipes - {}", localized_recipe.name).to_string(),
                            recipe: localized_recipe.clone(),
                            site: site.clone(),
                            flat_steps: localized_recipe.flat_steps(),
                            self_url: self_url.clone(),
                        },
                    )
                    .unwrap();

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
                        "{}{}/{}/",
                        site.public_url,
                        site_locale,
                        recipe.slug.clone().localized(Some(site_locale.clone()))?
                    ),
                });
            }
        }

        let index_html = handlebars
            .render(
                "link_list",
                &LinkListView {
                    locale: site_locale.clone(),
                    title: "Just Recipes - Home".to_string(),
                    links_label: "All Recipes".to_string(),
                    links: recipe_links,
                    site: site.clone(),
                    self_url: format!("{}{}/", &site.public_url, &site_locale),
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

        write_indexes(
            &handlebars,
            &Path::new(&locale_root).join("categories"),
            Url::parse(&site.public_url)?
                .join(&format!("{}/", site_locale))?
                .join("categories/")?,
            String::from("categories"),
            site.clone(),
            site_locale,
            categorized_recipes,
            &mut site_links,
        )?;

        write_indexes(
            &handlebars,
            &Path::new(&locale_root).join("cuisines"),
            Url::parse(&site.public_url)?
                .join(&format!("{}/", site_locale))?
                .join("cuisines/")?,
            String::from("cuisines"),
            site.clone(),
            site_locale,
            cuisine_recipes,
            &mut site_links,
        )?;
    }

    let home_html = handlebars
        .render(
            "index",
            &HomeView {
                locales: site_locales.to_vec(),
                title: "Just Recipes - Home".to_string(),
                site: site.clone(),
                self_url: site.public_url.clone(),
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
                self_url: format!("{}about/", &site.public_url),
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
                links: site_links.into_iter().collect(),
                site: site.clone(),
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

#[allow(clippy::too_many_arguments)]
fn write_indexes(
    handlebars: &Handlebars,
    base_dir: &Path,
    base_url: Url,
    group_type: String,
    site: SiteView,
    locale: &str,
    grouped_recipes: HashMap<String, Vec<(String, String)>>,
    site_links: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    let mut group_links: Vec<(String, String)> = Vec::new();

    for group in grouped_recipes.keys().sorted() {
        let group_slug: String = slugify!(group);
        let self_url = base_url.join(&format!("{}/", &group_slug))?;

        group_links.push((self_url.to_string(), group.clone()));
        site_links.insert(self_url.to_string());

        if let Some(links) = grouped_recipes.get(group) {
            let html = handlebars.render(
                "link_list",
                &LinkListView {
                    locale: locale.to_string(),
                    title: format!("Just Recipes - {} - {}", title(&group_type), title(group)),
                    links_label: title(group),
                    links: links.clone(),
                    site: site.clone(),
                    self_url: self_url.to_string(),
                },
            )?;

            let destination = Path::new(&base_dir).join(group_slug).join("index.html");
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::write(&destination, html)?;
        }
    }

    site_links.insert(base_url.to_string());

    let index_html = handlebars.render(
        "link_list",
        &LinkListView {
            locale: locale.to_string(),
            title: format!("Just Recipes - {}", title(&group_type)),
            links_label: title(&group_type),
            links: group_links,
            site,
            self_url: base_url.to_string(),
        },
    )?;

    let index_destination = Path::new(&base_dir).join("index.html");
    fs::write(&index_destination, index_html)?;
    Ok(())
}

/// Title case a string.
fn title(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
