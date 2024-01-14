/// aggregation worker that is in the background process

const modulePath = "/static/js/wasm/wasm_module.js";
let _dataset;
let _module;

self.onmessage = (msg) => {
  if (msg.data == "init") {
    // import main, { Dataset } from ;
    // load dynamically
    (async function init() {
      try {
        let module = await import(modulePath);
        await module.default();
        module.initSync(module);

        _module = module;
        _dataset = new _module.Dataset();

        self.postMessage({ op: "init" });
      } catch (error) {
        console.error("Init failed", error);
        self.postMessage({ op: "init", error: error });
        return;
      }
    })();
  } else if (msg.data == "fetch") {
    (async function fetch() {
      try {
        let rowsCount = await _dataset.fetch_data();
        self.postMessage({ op: "fetch", res: rowsCount });
      } catch (error) {
        self.postMessage({ op: "fetch", error: error });
      }
    })();
  } else if (msg.data == "aggregate1") {
    let task = new _module.AggregateTask(_module.AggregateMethod.MaxSum);
    (async function run() {
      try {
        const aggregated = await _dataset.aggregate_data(task);
        self.postMessage({ op: "aggregate1", res: aggregated });
      } catch (error) {
        console.error("Aggregate failed", error);
        self.postMessage({ op: "aggregate1", error: error });
      }
    })();
  } else if (msg.data == "aggregate2") {
    const task = new _module.AggregateTask(_module.AggregateMethod.MinSum);
    try {
      const aggregated = _dataset.aggregate_data(task);
      self.postMessage({ op: "aggregate2", res: aggregated });
    } catch (error) {
      console.error("Aggregate failed", error);
      self.postMessage({ op: "aggregate2", error: error });
    }
  } else {
    self.postMessage({ op: msg.data, error: "Unknown command" });
  }
};
