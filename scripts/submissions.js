let fs = require("fs")

let files = fs.readdirSync("./submissions");

let jsons = files.map(x => {
    console.log(x)
    return JSON.parse(fs.readFileSync("./submissions/" + x).toString('utf-8'))
})

let jsons2 = jsons.map(x => {
    let tag = "❌";

    let participant = x.tag == "Err" ? x.data[1].name : x.data.participant.name;

    let log = x.tag == "Ok" ? x.data.log.join("") : x.data[0].substring(0, 200);

    if (x.tag == "Ok" && x?.data?.data["compile"]) {
        tag = "🟧"
    }

    if (x.tag == "Ok" && x?.data?.data["fibbo"]) {
        tag = "✅"
    }

    if (x.tag == "Ok" && x?.data?.data["timeout"]) {
        tag += " 🕑"
    }

    log = log.replaceAll("\n", " ")

    return {
        tag, participant, log
    }
})

console.log("| name | status | log |\n| :-: | :-:| :- |")
console.log(jsons2.map(x => "| " + x.participant + " | " + x.tag + " | " + (x.log.substring(0, 200) + (x.log.length > 100 ? "..." : "")) + " | ").join("\n"))