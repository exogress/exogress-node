const Instance = require(".");

let instance = new Instance({
    access_key_id: "01F68JEA8XW0MM1XGGR47F7KSD",
    secret_access_key: "a83Xj28xao6UkHRasZUhVVrrhc26w8RMJsyV7kkgn7jU",
    account: "glebpom",
    project: "location-tester",
    watch_config: true,
    config_path: "./Exofile.yml",
    labels: {
        label1: "val1",
    },
    profile: "asd"
});

instance.spawn().then(function(res) {
    console.log(res);
});

const http = require('http');

const hostname = '127.0.0.1';
const port = 4000;

const server = http.createServer((req, res) => {
    res.statusCode = 200;
    res.setHeader('Content-Type', 'text/plain');
    res.end('Hello, World!\n');
});

server.listen(port, hostname, () => {
    console.log(`Server running at http://${hostname}:${port}/`);
});

