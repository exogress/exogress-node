"use strict";

const { promisify } = require("util");

const { instanceNew, instanceStop, instanceReload, instanceSpawn } = require("../native/index.node");

const instanceSpawnAsync = promisify(instanceSpawn);

class Instance {
    constructor(args) {
        this.instance = instanceNew(args);
    }

    spawn() {
        return instanceSpawnAsync.call(this.instance);
    }

    reload() {
        instanceReload.call(this.instance);
    }

    stop() {
        instanceStop.call(this.instance);
    }
}

module.exports = Instance;
