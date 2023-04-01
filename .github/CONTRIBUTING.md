# Contributing
This guide outlines how to create a new Aidoku source. Please read it **carefully** before you get started, or if you don't have any experience on the required languages and tooling.

This guide is not definitive by any means. If you find any issue, report it by [creating an issue](https://github.com/Skittyblock/aidoku-community-sources/issues/new) or submit a pull request directly.

## Table of contents
1. [Prerequisites](#prerequisites)
2. [Getting help](#getting-help)
3. [Writing a source](#writing-a-source)
    1. [File structure](#file-structure)
    2. [Dependencies](#dependencies)
    3. [Exported functions](#exported-functions)
    4. [Notes](#notes)
4. [Template sources](#template-sources)
    1. [Directory structure](#directory-structure)
5. [Running](#running)
6. [Debugging](#debugging)
    1. [Good old print statements](#good-old-print-statements)
    2. [Inspecting network calls](#inspecting-network-calls)
7. [Submitting the changes](#submitting-changes)
    1. [Pull Request checklist](#pull-request-checklist)

## Prerequisites
Before you start, please note that basic knowledge in these technologies is **required**.
- The [Rust](https://doc.rust-lang.org/book/) programming language
- Web scraping:
  - [HTML](https://developer.mozilla.org/en-US/docs/Web/HTML)
  - [CSS selectors](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors)

### Tooling
- An iOS device/simulator with a recent version of Aidoku
- [aidoku-cli](https://github.com/Aidoku/aidoku-cli) (optional, recommended if you're not developing on a macOS machine)
- Basic image editing knowledge

### Cloning the repository
Some steps can be taken to ignore the `gh-pages` branch and skip unrelated sources, which reduces time taken to pull and navigate. This will also reduce disk usage and network traffic.

<details>
    <summary>Steps</summary>

1. Delete the `gh-pages` branch in your repo. You may also want to disable GitHub Actions in the repo settings.
2. Do a partial clone:
```sh
git clone --filter=blob:none --no-checkout <fork-repo-url>
cd aidoku-community-sources/
```
3. Enable sparse checkout:
```sh
git sparse-checkout set
```
4. Edit `.git/info/sparse-checkout` to filter checked out paths. Here's an example:
```sh
/*
!/src

# (optional) ignore the C bindings if you're not working with C 
!/lib

# Allow a single source
/src/<lang>/<source>
```
5. Configure remotes:
```sh
git remote add upstream https://github.com/Skittyblock/aidoku-community-sources

# Optionally disable pushing to upstream
git remote set-url --push upstream no_pushing

# Only fetch upstream's main branch
git config remote.upstream.fetch "+refs/heads/main:refs/remotes/upstream/main"

# Update remotes
git remote update

# Track upstream's main branch
git branch main -u upstream/main

# Switch to main branch
git switch main
```
6. Optional useful configuration:
```sh
# prune obsolete remote branches on fetch
git config remote.origin.prune true

# fast-forward only when pulling master branch
git config pull.ff only
```

If you change the sparse checkout filter later, run `git sparse-checkout reapply`.

Read more on [partial clone](https://github.blog/2020-12-21-get-up-to-speed-with-partial-clone-and-shallow-clone/), [sparse checkout](https://github.blog/2020-01-17-bring-your-monorepo-down-to-size-with-sparse-checkout/) and [negative refspecs](https://github.blog/2020-10-19-git-2-29-released/#user-content-negative-refspecs).
</details>

## Getting help
- Join [the Discord server](https://discord.com/invite/9U8cC5Zk3s) to ask questions while developing your source. Please do so in the `#source-dev` channel.

## Writing a source
The fastest way to get started is to either copy an existing source and renaming as needed, or using `aidoku init` from [aidoku-cli](https://github.com/Aidoku/aidoku-cli). You should also read through a few existing sources' code before you start.

Each source should be in `src/<lang>/<source-language>.<source-name>`:
- The `<lang>` part is the programming language your source is developed in, so `rust` for Rust.
- `<source-language>`: Use `multi` if your source supports multiple languages, or omit if it's a template source.
`<source-language>` should contain the full locale string in lowercase. For example, if you're creating a Portugese (Brazilian) source, use `pt-br`.

### File structure
A simple Rust source structure looks like this:
```sh
$ tree src/rust/<sourcename>/
src/rust/<sourcename>/   
├── build.ps1
├── build.sh
├── Cargo.lock
├── Cargo.toml
├── res
│   ├── filters.json
│   ├── Icon.png
│   └── settings.json
│   └── source.json
└── src
    ├── helper.rs
    └── lib.rs
```

#### source.json
A minimal manifest which describes the source to Aidoku. Make sure it follows this structure:
```json
{
    "info": {
        "id": "<lang>.<sourcename>",
        "lang": "<lang>",
        "name": "<My source name>",
        "version": 1,
        "url": "<Source URL>",
        "urls": [
            "<Source URL>",
            "<Additional source URLs>"
        ],
        "nsfw": 1
    },
    "languages": [
        { "code": "<lang>" },
        { "code": "<lang2>" },
    ],
    "listings": [
        { "name": "<Listing name>" }
    ]
}
```
| Field          | Description                                                                                                                        |
|----------------|------------------------------------------------------------------------------------------------------------------------------------|
| `info.id`      | A unique identifier for the source. The language and the site name should be enough.                                               |
| `info.lang`    | The source's language.                                                                                                             |
| `info.name`    | The displayed name of the source.                                                                                                  |
| `info.version` | The source's version number. It must be a positive integer and incremented with any notable changes.                               |
| `info.url`     | The source's main URL, which can be used for [deep linking](#handle_url).                                                        |
| `info.urls`    | Any additional URLs used for [deep linking](#handle_url). If `info.url` isn't available, the first item is used as the main URL. |
| `info.nsfw`    | The NSFW level of the source. `0` for sources with no NSFW content at all, `1` for some NSFW, and `2` for majority NSFW sources.   |
| `languages`    | An array of languages that the source supports. Use on multi-language sources only.                                                |
| `listings`     | An array of listings that the source supports outside of the default `All` listing. These extra listings are not filterable.       |

#### settings.json
Expose user settings by declaring them in this file. See this [gist](https://gist.github.com/beerpiss/18e0a861a55f6fa9ed6733798f027ee0) for more information.

#### filters.json
Aidoku supports search filters which are declared in this file. 

### Dependencies
Sources rely on bindings which are in the [aidoku-rs](https://github.com/Aidoku/aidoku-rs) crate. Detailed documentation can be found [here](https://aidoku.github.io/aidoku-rs/aidoku/).

### Exported functions
#### `initialize`
Called once on source startup. Use it to do any initialization work (e.g. setting the rate limit).

#### `get_manga_list`
a.k.a the All listing.

- The app calls `get_manga_list` which should return a `MangaPageResult` containing a batch of found `Manga` entries.
  - This function supports pagination. When the user finishes scrolling the first batch and more results need to be fetched, the app calls this function again with increasing `page` values starting from `1`. This continues until the source returns `MangaPageResult.has_more` as `false`.
  - When the user searches inside the app, this function will be called with an array of filters passed. If search functionality is not available, remove the [`filters.json`](#filtersjson) file.
- To show the list properly, the app needs at least `Manga.title` and `Manga.id` set. The rest of the fields can be filled later.

#### `get_manga_listing`
Invoked for any other listing outside "All". Similar process to `get_manga_list`, but does not support filtering.

#### `get_manga_details`
When the user taps on a manga, this function will be called with the manga's ID to update a manga's details.
- If a manga is cached, this function is invoked if the user does a manual update (pull-to-refresh).

#### `get_chapter_list`
Function is called to display the chapter list after `get_manga_details` is done. **The list should be sorted descending by the source order.**

- `Chapter.date_updated` is the [UNIX epoch time](https://en.wikipedia.org/wiki/Unix_time) expressed in **seconds**.
    - If this attribute is less than or equal to zero, the app won't display date updated.
    - To get the time in seconds from a date string, use `ValueRef.as_date` (Rust) or `ValueRef.toDate` (AssemblyScript). The date format must be compatible with [NSDateFormatter](https://nsdateformatter.com/).
    - If there is any problem parsing, return `-1`.

#### `get_page_list`
- When the user opens a chapter, `get_page_list` will be called and it should return a list of `Page`s.
- Currently, the source must provide all the page images directly, and they **must be set as an absolute URL**.
- Chapter page numbers start from 0.

#### `handle_url`
When a source URL is opened with Aidoku, this function will be called and it returns a `DeepLink` object containing the manga and chapter that the URL pointed to.

#### `handle_notification`
Used for handling any [setting](#settingsjson) changes.

### Notes
- You probably want to percent-encode any user input that goes in the URL (e.g. title search). If you scaffolded a source using aidoku-cli, there's already a helper function called `urlencode`. If there isn't, check existing sources for an implementation.

## Template sources
Multiple source sites may use the same site generator tool (usually a content management system), and so it makes sense to reuse code.

The **template** contains the base implementation, and then each source defines its own implementation upon that base, which then is used to generate individual sources from.

### Directory structure
```sh
$ ls src/rust/<template>
src/rust/<template>
├── build.ps1
├── build.sh
├── res
├── sources
│   └── <sourcename>
└── template
    └── src
        ├── template.rs
        ├── lib.rs
        └── helper.rs
```
- `build.ps1` and `build.sh` are the template build scripts
  - Call them with no arguments (or `-a`) to build all sources
  - Call them with a `<sourcename>` to build the source package for that website.
- `<template>/template` defines the template's default implementation.
- `res` is the template's default resources (filters, icons, etc.). If a source doesn't have their own resources, then the default will be used.
- `sources` are the implementations for sources using the template.

## Running
To make development more convenient on non-Apple devices, you can use `aidoku serve` from [aidoku-cli](https://github.com/Aidoku/aidoku-cli) to create a local source list:
```sh
$ aidoku serve *.aix
Listening on these addresses:
  # A list of IP addresses will be displayed here.
  # Pick the one which looks the most like your local network's IP address 
  # and add it to Aidoku (assuming your development machine and your device
  # is on the same network).
  #
  # To make picking addresses easier, addresses that looks like your local IP
  # address are highlighted in green.
Hit CTRL-C to stop the server
```

## Debugging
### Good old print statements
You can use `aidoku::prelude::println!` (Rust) or `console.log` (AssemblyScript) to print messages that will show up in Aidoku logs (Aidoku -> Settings -> Display Logs). These logs also show up in the console if you're running Aidoku in the Xcode simulator.

Alternatively, use `aidoku logcat` from [aidoku-cli](https://github.com/Aidoku/aidoku-cli) to stream logs to your development machine:
```sh
$ aidoku logcat
Listening on these addresses:
  # A list of IP addresses will be displayed here.
  # Pick the one which looks the most like your local network's IP address 
  # and edit the Log Server option in Aidoku with that address.
  #
  # To make picking addresses easier, addresses that looks like your local IP
  # address are highlighted in green.
```

### Inspecting network calls
If you want to take a look into the network flow, you can use a web debugging tool.

#### Setup your proxy server
We are going to use [mitmproxy](https://mitmproxy.org/) but any web debugger (e.g. Charles, Fiddler) is fine. To install and execute, run the commands below:
```sh
# Install the tool. If you don't have Python 3 installed, do it first.
$ sudo pip3 install mitmproxy
# Execute the web interface and the proxy.
$ mitmweb
```
After running, navigate to http://localhost:8081 from your browser.

Then edit your device's network settings by navigating to Settings -> Wi-Fi -> `<your network name>` -> Proxy and set configuration to Manual. Set the address to your machine's address, and the port to 8081.
    
Afterwards, visit [mitm.it](http://mitm.it) to install the mitmproxy certificate, which is required when proxying HTTPS traffic. Refer to [this link](https://docs.mitmproxy.org/stable/concepts-certificates/) for more details.

If all went well, you should see all requests and responses made by the source in the web interface of `mitmweb`.

## Submitting changes
When you feel confident about your changes, submit a new Pull Request so your code can be reviewed and merged if it's approved. We encourage following a [GitHub Standard Fork & Pull Request Workflow](https://gist.github.com/Chaser324/ce0505fbed06b947d962) and following the good practices of the workflow, such as not commiting directly to `main`: always create a new branch for your changes.

Please test your changes by running the source on a simulator or a test device before submitting it. Also make sure to follow the PR checklist available in the PR body field when creating a new PR. As a reference, you can find it below.

### Pull Request checklist
Checklist:
- [ ] Updated source's version for individual source changes
- [ ] Updated all sources' versions for template changes
- [ ] Set appropriate `nsfw` value
- [ ] Did not change `id` even if a source's name or language were changed
- [ ] Tested the modifications by running it on the simulator or a test device 
