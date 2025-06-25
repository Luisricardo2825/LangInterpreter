// fibonnacci recursivo
function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
let a = 1476;
Io.println(fibonacci(a));
