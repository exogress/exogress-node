const { promisify } = require('util');

const exogress = require('../native');
exogress.spawnP = promisify(exogress.spawn);

console.log(
    exogress.spawnP({
        client_id: "",
        client_secret: "",
        project: "",
        account: "",
    })
        .then(console.log)
        .catch(error => { console.log('Exogress error: ', error.message); })
);

const express = require('express')
const app = express()
const port = 4000

app.get('/', (req, res) => res.send('<html><body><h1>Hello from JS</h1></body></html>'))
app.listen(port, () => console.log(`Example app listening at http://localhost:${port}`))
