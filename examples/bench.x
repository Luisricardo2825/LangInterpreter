let bilion = 10_000_000_000
let million = 1_000_000

fn iter_exaust(length){
    let count = 0
    for (let i = 0; i < length; i = i + 1) { 
        count = count+1
    }
    return count
}

fn toSeconds(ms){
    return ms/1000
}

fn bench(func,param){
    let start = now()
    let result = func(param)
    let end = now()
    println("Result: ",result," final: ",toSeconds(end-start))
}



bench(iter_exaust,10000)

// bench(iter_exaust,1_000_000_000)