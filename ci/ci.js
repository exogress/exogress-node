const fs = require('fs');
const pjson = require('../package.json');
var path = require("path");

const stage = path.join("build", "stage", "v" + pjson.version);

const cmd = process.argv[2];

if (cmd === "check_tag") {
    const tag = process.argv[3];
    let expectedTag = "v" + pjson.version;
    
    if (expectedTag !== tag) {
        console.log("Version mismatch!");
        process.exit(1)
    } else {
        process.exit(0)
    }
}

fs.readdir(stage, (err, files) => {
    if (cmd === "asset_path") {
        let filePath = path.resolve(stage + "/" + files[0]);
        console.log(filePath);
    } else if (cmd === "asset_name") {
        console.log(files[0])
    }
    process.exit(0)
});
