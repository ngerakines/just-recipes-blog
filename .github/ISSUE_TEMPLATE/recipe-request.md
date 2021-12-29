---
name: Recipe request
about: Add a recipe
title: Recipe request
labels: 'recipe'
assignees: ''

---

The recipe:

```yaml
id: e8733aff-143c-445e-bac6-8d36c8500cf9
locales: ["en_US"]
name: A wonderful new recipe
slug: 8d36c8500cf9-a-wonderful-new-recipe
description: This recipe is pretty neat.
ingredients:
  - celery
  - onion
  - bell pepper
equipment:
  - dutch oven
stages:
  - name: Cook
    cook_time: 10 minutes
    prep_time: 5 minutes
    description: A block of text that appears before the steps of this stage.
    footer: A block of text that appears below the steps of this stage.
    steps:
      - First do this
      - Then do that
```

Content authors are strongly encouraged to add at least one image of the recipe.

# Checklist

This can be removed once everything has beek ticked.

* [ ] Does the recipe have a unique ID? https://www.uuidgenerator.net/version4
* [ ] Is the recipe URL slug correct and is it prefixed with the last section of the id? https://slugify.online/
* [ ] Does the recipe need a description?
* [ ] Are all of the ingredients listed?
* [ ] Area all pieces of non-standard equipment listed?
* [ ] Can the stages be condensed or do they need to be split up?
* [ ] Does each stage contain realistic `prep_time` and `cook_time` values?

# Images

* Is the image high-resolution?
* Is the subject of the image clear?
* Does the image leak any identifying or distracting content?
