class Number {
    value = null;
    constructor(value) {
        this.value = value;
    }

    static hello() {
        Io.println("Olá");
    }

    plus(other) {
        return new Number(this.value + other);
    }

    @Operator plus(other) {
        return new Number(this.value + other);
    }
    @Operator sub(other) {
        return new Number(this.value - other);
    }
    @Operator mul(other) {
        return new Number(this.value * other);
    }

    @Operator div(other) {
        return new Number(this.value / other);
    }

    @Operator mod(other) {
        return new Number(this.value % other);
    }

    valueOf() {
        return this.value;
    }
    toString() {
        return this.value + "";
    }
}
