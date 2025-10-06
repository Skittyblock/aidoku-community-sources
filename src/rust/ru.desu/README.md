### To update filters use following JS code in browser at [this page](https://desu.city/manga/):  
#### Note: this code will automatically copy a new filters JSON  

```js
let kinds = Array.from(document.querySelectorAll('ul[class="catalog-kinds"] > li > div')).map(x => {
    let id = x.querySelector('input[type="checkbox"]')?.dataset.kind;
    let name = x.querySelector('span[class="filter-control-text"]')?.innerText;
    return {
        type: "check",
        name,
        id: `0|${id}`
    };
});
let cats = Array.from(document.querySelectorAll('ul[class="catalog-genres"] > li > div')).map(x => {
    let checkBox = x.querySelector('input[type="checkbox"]');
    let isTag = x.querySelector('span[class="filter-control-text"] > span')?.innerText == '#';
    let id = checkBox.dataset.genreSlug;
    let name = checkBox.dataset.genreName;
    return {
        type: "genre",
        isTag,
        name,
        id,
        canExclude: false
    }
});
let status = Array.from(document.querySelectorAll('ul[class="catalog-status"] > li > div')).map(x => {
    let id = x.querySelector('input[type="checkbox"]')?.dataset.status;
    let name = x.querySelector('span[class="filter-control-text"]')?.innerText;
    return {
        type: "check",
        name,
        id: `1|${id}`
    }
});
let filtersObj = [
    {
        type: "title"
    },
    {
        type: "group",
        name: "Статус",
        filters: status
    },
    {
        type: "group",
        name: "Тип",
        filters: kinds,
    },
    {
        type: "group",
        name: "Жанры",
        filters: cats.filter(x => !x.isTag)
            .map(x => { return { type: x.type, name: x.name, id: x.id, canExclude: x.canExclude } })
    },
    {
        type: "group",
        name: "Теги",
        filters: cats.filter(x => x.isTag)
            .map(x => { return { type: x.type, name: `#${x.name}`, id: x.id, canExclude: x.canExclude } })
    },
    {
        type: "sort",
        name: "Упорядочить",
        canAscend: false,
        options: [
            "По добавлению",
            "По алфавиту",
            "По популярности",
            "По обновлению"
        ],
        "default": {
            "index": 3
        }
    }
];

copy(JSON.stringify(filtersObj, null, 4) + '\n');
```