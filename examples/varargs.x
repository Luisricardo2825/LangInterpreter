class Num {
    value;
    constructor(value) {
        this.value = value;
    }
    valueof() {
        return this.value;
    }
}

function treatArgs(...args) {
    for (let i = 0; i < args.length(); i++) {
        if (args[i] instanceof String) {
            Io.println(typeof args[i], args[i], "é String");
        } else if (args[i] instanceof Number) {
            Io.println(typeof args[i], args[i], "é Number");
        } else if (args[i] instanceof Array) {
            Io.println(typeof args[i], args[i], "é Array");
        } else {
            Io.println(args[i], "é deconhecido");
        }
    }
}
let num1 = new Num(10);

// let n = new Teste("Ricardo", 10);
treatArgs("a", "b", "c", 1, 2, 3, num1.valueof(), [1, 2, 3]);
