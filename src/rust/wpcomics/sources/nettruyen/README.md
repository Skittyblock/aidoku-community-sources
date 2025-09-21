script get categories:
```js
JSON.stringify($(temp1).find("option").toArray().map(option => {
  return { type: "genre", id: $(option).val().split("/").at(-1), name: $(option).text().trim() }
}))
```