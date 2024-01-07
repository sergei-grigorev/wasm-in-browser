# wasm-in-browser
Experiments with running WASM inside the browser and on serverless.

## wasm-server
HTTP Server can also be WASM, why not? With the [Spin SDK](https://www.fermyon.com) I actually build and run light WASM server. Use `just` command to build and start.

## wasm-module
That is the project to build wasm file and js. That module is downloaded to the browser and aggregate Apache Arrow dataset that web-server serves. Use `just` command to build and start.

## run
I use [Just](https://just.systems) to help me run commands more easier. You can copy commands from it or run using `just`. Also you need to install [watch](https://watchexec.github.io)
that watches wasm-module changes and runs re-deploy.
- call `just watch-wasm` to monitor wasm-module changes, it will compile and copy each your change to web-server assets directory
- call `just spin-watch` to run web-server that will be updated each time you update assets (wasm-module) or wasm-server

```
Serving http://127.0.0.1:3000
Available Routes:
  data: http://127.0.0.1:3000/data
  index-redirect: http://127.0.0.1:3000
  static: http://127.0.0.1:3000/static (wildcard)
```
