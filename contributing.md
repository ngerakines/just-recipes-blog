# Contributing

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](code_of_conduct.md) 

All are welcome to contribute to `jrb` and whether it is code that supports, enhances, and secures the jrb platform, or if it recipe or site content, we welcome and appreciate your time, effort, and energy.

# Recipes

Recipes are stored in the `./recipes/` directory as yaml files. Each recipe is saved as an individual yaml file.

## Adding Recipes

The "easy" way to create a new recipe is to use the `jrb` tool to initalize one. See `jrb --help` and `jrb init --help` for more information.

Alternatively, follow these instructions for each recipe that you would like to add:

1. Fork https://github.com/ngerakines/just-recipes-blog on GitHub
1. Visit https://www.uuidgenerator.net/ and generate a new "version 4" UUID. It will look like "6bf61236-2fe6-4244-8255-7899f6e9ddb6"
1. Copy the last block of the UUID and create a file in the "`./recipes/`" directory with with that block of characters and the first few words of the recipe. The file extension must be "`.yml`". For example, with the above UUID and the recipe name "Steamed rice", the file name should be "`7899f6e9ddb6-steamed-rice.yml`".
1. Create a pull-request for the recipe to be reviewed and merged in.

## Localization

The base language for the website is English and all localized content defaults to English content when translation strings are not present.

When localizing a recipe, the following checklist may help:

* [ ] `name` Recipe name
* [ ] `slug` The unique URL for the recipe
* [ ] `description` The description of the recipe
* [ ] `tags` A list of tags that can help categorize and label the recipe
* [ ] `ingredients` A list of materials and ingredients that make up the recipe
* [ ] `equipment` A list of equipment and materials that are used to product the thing being cooked
* [ ] `stages.*.name` The name of each stage in the recipe
* [ ] `stages.*.description` Each description content for each stage
* [ ] `stages.*.steps.*` Each step of each stage in the recipe
* [ ] `stages.*.footer` Each footer content for each stage
