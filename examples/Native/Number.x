export class Number {
    value = null;
    constructor(value) {
        this.value = value;
    }

    static hello(){
        Io.println("Olá")
    }
    
    plus(other){
        return new Number(this.value + other);
    }

    operator plus(other) {
        return new Number(this.value + other);
    }
    operator sub(other) {
        return new Number(this.value - other);
    }
    operator mul(other) {
        return new Number(this.value * other);
    }

    operator div(other) {
        return new Number(this.value / other);
    }
    
    operator mod(other) {
        return new Number(this.value % other);
    }
    
    valueOf() {
        return this.value;
    }
    toString() {
        return this.value + "";
    }
}
