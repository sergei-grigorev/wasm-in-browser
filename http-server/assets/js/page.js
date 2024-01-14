// script that updates the index.html

const logBlock = document.querySelector("ul#aggregate_log");

/**
 * Add new line to the progress.
 *
 * @param {string} message
 */
function addMessage(message) {
  let li = document.createElement("li");
  li.className = "list-group-item";

  let p = document.createElement("p");
  p.innerText = message;
  li.appendChild(p);

  logBlock.appendChild(li);
}

/**
 * Update the progress bar.
 *
 * @param {string} message - message that will be added to the existed
 * @param {number} opTimeMs - request processing time
 */
function updateLastMessage(message, opTimeMs) {
  let li = logBlock.lastChild;
  let p = li.lastChild;
  p.innerText += " " + message;

  let em = document.createElement("em");
  const requestTimeText = " (" + Math.fround(opTimeMs) + " ms)";
  em.textContent = requestTimeText;
  p.appendChild(em);
}

/**
 * Init data aggregation.
 */
function initUI() {
  const worker = new Worker("./js/aggregate_worker.js");

  // trace request time
  let lastMessageMs;
  const startTask = (caption, name) => {
    addMessage(caption);
    lastMessageMs = performance.now();
    worker.postMessage(name);
  };

  worker.onmessage = (msg) => {
    const query = msg.data.op;
    const currentTime = performance.now();
    const opTimeMs = currentTime - lastMessageMs;

    if (query == "init") {
      updateLastMessage("done", opTimeMs);
      startTask("Fetch data ...", "fetch");
    } else if (query == "fetch") {
      if (msg.data.res) {
        updateLastMessage(msg.data.res + " rows ", opTimeMs);
        startTask("Aggregate data 1 ...", "aggregate1");
      } else {
        console.error(msg.data.error);
        worker.terminate();
        updateLastMessage("failed fetching data", opTimeMs);
      }
    } else if (query == "aggregate1") {
      if (msg.data.res) {
        updateLastMessage(msg.data.res, opTimeMs);
        startTask("Aggregate data 2 ...", "aggregate2");
      } else {
        console.error(msg.data.error);
        worker.terminate();
        updateLastMessage("aggregate failed", opTimeMs);
      }
    } else if (query == "aggregate2") {
      if (msg.data.res) {
        updateLastMessage(msg.data.res, opTimeMs);
      } else {
        console.error(msg.data.error);
        updateLastMessage("aggregate failed", opTimeMs);
      }

      worker.terminate();
    } else {
      console.error(msg.data);
      worker.terminate();
      updateLastMessage("worker failed");
    }
  };

  startTask("Load WASM module ...", "init");
}

export { initUI };
