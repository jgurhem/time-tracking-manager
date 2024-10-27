# Time tracking manager

This tool is designed to manage time tracking tools and convert from one format to another, alleviating the need to track time on multiple platforms.

## How to install the tool ?

## Local compilation

Dependencies:

- Rust

Clone the repository and within the folder execute:

```bash
cargo build --release
```

The application can be used with:

```bash
target/release/time-tracking-manager --help
```

## Installation in the web browser from release

Dependencies:

- Tampermonkey or equivalent

You need to create a tampermonkey script with the following header:

```js
// ==UserScript==
// @name         Time tracking manager
// @namespace    http://tampermonkey.net/
// @version      0.1.0
// @description  Convert time tracking elements from a given provider into Progessi
// @author       Jérôme Gurhem (https://github.com/jgurhem)
// @match        https://aneo.progessi.com/home/control/timesheetByLine**
// @grant        GM_getResourceURL
// @require      https://github.com/jgurhem/time-tracking-manager/releases/download/0.1.0/time_tracking_manager.js
// @resource     wasm https://github.com/jgurhem/time-tracking-manager/releases/download/0.1.0/time_tracking_manager.wasm
// @connect      api.clockify.me
// ==/UserScript==
```

The match option configures the script to run on the Progessi Website.
Then, the following code should be added to load the Webassembly application into Progessi Website:

```js
(async function () {
    const url = GM_getResourceURL("wasm").replace('data:application;base64,','data:application/wasm;base64,');
    // use wasm pack functions to load our WebAssembly
    const { ProgessiHandle } = wasm_bindgen;
    await wasm_bindgen(fetch(url));

    setTimeout(async () => {
        var pro = await ProgessiHandle.new({provider: "clockify",
                                            provider_options: ["token=<You clockify token>"],
                                           },
                                           document);

    }, 50)
})();
```

This code extracts the ProgessiHandle from the WebAssembly and load it within the current context so that it can be used.
Then, we create a new instance of the handle that provides the services to extract time table from the selected provider (here: clockify) and import them in Progessi.

## Installation in the web browser from local sources

Dependencies:

- Tampermonkey or equivalent
- Configure web browser (Chrome based) to allow access to file URLs for Tampermonkey extension
- Rust
- Wasm-pack `cargo install wasm-pack`

You need to clone the project and go in its directory.
To build the WebAssembly for local use, execute the following command:

```bash
wasm-pack build --no-typescript --target no-modules
```

Build outputs can be found in the `pkg` folder.
They can be used in place of the release if you edit your Tampermonkey header as follow:

```js
// ==UserScript==
// @name         Time tracking manager
// @namespace    http://tampermonkey.net/
// @version      0.1.0
// @description  Convert time tracking elements from a given provider into Progessi
// @author       Jérôme Gurhem (https://github.com/jgurhem)
// @match        https://aneo.progessi.com/home/control/timesheetByLine**
// @grant        GM_getResourceURL
// @require      file://<PATH-TO-THE-PROJECt>\pkg\time_tracking_manager.js
// @resource     wasm file://<PATH-TO-THE-PROJECt>\pkg\time_tracking_manager_bg.wasm
// @connect      api.clockify.me
// ==/UserScript==
```
