// script that updates the index.html

const logBlock = document.querySelector("p#aggregate_log");
function printProgress(message) {
    logBlock.innerHTML += message + "<br/>";
}

/// Init UI
function initUI() {
    const worker = new Worker("./js/aggregate_worker.js");
    worker.onmessage = (msg) => {
        const query = msg.data.op;
        if (query == "init") {
            printProgress("Fetch data ...");
            worker.postMessage("fetch");
        } else if (query == "fetch") {
            if (msg.data.res) {
                printProgress("Successfully fetched data: " + msg.data.res + " rows");
                printProgress("Aggregate data 1 ...");
                worker.postMessage("aggregate1");
            } else {
                console.error(msg.data.error);
                worker.terminate();
                printProgress("failed fetching data");
            }
        } else if (query == "aggregate1") {
            if (msg.data.res) {
                printProgress("done: " + msg.data.res);
                printProgress("Aggregate data 2 ...");
                worker.postMessage("aggregate2");
            } else {
                console.error(msg.data.error);
                worker.terminate();
                printProgress("aggregate failed");
            }
        } else if (query == "aggregate2") {
            if (msg.data.res) {
                printProgress("done: " + msg.data.res);
                worker.postMessage("aggregate2");
            } else {
                console.error(msg.data.error);
                printProgress("aggregate failed");
            }

            worker.terminate();
        } else {
            console.error(msg.data);
            worker.terminate();
            printProgress("worker failed");
        }
    };

    printProgress("Load WASM module");
    worker.postMessage("init");
}

export {  initUI };
