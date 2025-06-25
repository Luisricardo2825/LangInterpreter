// fibonnacci recursivo
let console = { log: Io.println };

function fibonacci(n) {
    let a = 0;
    let b = 1;
    for (let i = 0; i < n; i++) {
        let arr = [b, a + b];
        a = arr[0];
        b = arr[1];
    }
    return a;
}

console.log(fibonacci(100)); // 6
