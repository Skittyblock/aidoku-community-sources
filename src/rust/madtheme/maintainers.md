### quick way to get genre list
open search page, run this in devtools console, copy and paste.

```js
JSON.stringify([...document.querySelectorAll("div.checkbox-group.genres label.checkbox")].map(e => ({ type: "genre", name: e.querySelector(".radio__label").innerText, id: e.querySelector("input").getAttribute("value"), canExclude: false })))
```
