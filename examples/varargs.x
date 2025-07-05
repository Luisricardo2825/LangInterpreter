class Num {
    value;
    constructor(self, value) {
        self.value = value;
    }
    valueOf(self) {
        return self.value;
    }
    toString(self) {
        return self.value;
    }
}

function treatArgs(...args) {
    for (let arg of args) {
        if (arg instanceof String) {
            Io.println(typeof arg, arg, "é String");
        } else if (arg instanceof Number) {
            Io.println(typeof arg, arg, "é Number");
        } else if (arg instanceof Num) {
            Io.println(typeof arg, arg, "é Num");
        } else if (arg instanceof Array) {
            Io.println(typeof arg, arg, "é Array");
        } else {
            Io.println(arg, "é deconhecido");
        }
    }
}
let num1 = new Num(10);

// let n = new Teste("Ricardo", 10);
treatArgs("a", "b", "c", 1, 2, 3, num1, [1, 2, 3]);
