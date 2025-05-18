let bilion = 10_000_000_000
let million = 1_000_000

fn iter_exaust(length){
    let count = 0
    for (let i = 0; i < length; i = i + 1) { 
        count = count+1
    }
    println(count)
}

let start = now()
iter_exaust(100_000)
let end = now()

println("final:",toSeconds(end-start))