const { promisify } = require('util');

const exogress = require('../native');
exogress.spawnP = promisify(exogress.spawn);
console.log(
    exogress.spawnP({
        access_key_id: "01EJ739PD8MEJXPQP89NCTJT80",
        secret_access_key: "2eoAYVjpjtztomf7mL94fJeZVS5TSkvEDSYB97v1CQxDDyfeg2GGiJLqDfbHTpMjcZK2KhmvoyjS1e3ssdzTdxqMoGDJbDypTnr4cHuKP8AhV1BkeTeeC7TK5HEEVZqAHJ4eeBicMeHEbpPfaQuaTKCiR3xEzSYCtQSroqiaDd9E2PbAYyBEjXxH6gC8f",
        account: "glebpom",
        project: "home",
    })
        .then(console.log)
        .catch(error => { console.log('Exogress error: ', error.message); })
);

const express = require('express')
const app = express()
const port = 4000

app.get('/', (req, res) => res.send('<html><body><h1>Hello from JS</h1></body></html>'))
app.listen(port, () => console.log(`Example app listening at http://localhost:${port}`))
