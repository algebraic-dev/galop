let fs = require("fs")

let files = fs.readdirSync("./submissions");

let jsons = files.map(x => 
    JSON.parse(fs.readFileSync("./submissions/" + x).toString('utf-8')));

console.log(JSON.stringify(jsons, null, 4))