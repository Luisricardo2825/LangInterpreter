class Object1 {
    constructor() {
        Io.println("Object1 constructor");
    }
    static ola() {
        Io.println("Object1 ola");
    }
}

class Object2 extends Object1 {
    static ola() {
        Io.println("Object2 ola");
    }
}

let a = new Object2();

Io.println("Object1 is instanceof Object2?", a instanceof Object1);
