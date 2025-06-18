export class Set {
    value = [];
    size = 0;
    constructor(...values) {
        if (values.length() == 1 && values[0] instanceof Array) {
            this.value = values[0];
            this.size = this.value.length();
        } else {
            for (let item of values) {
                this.push(item);
            }
        }
    }

    push(value) {
        if (!this.has(value)) {
            this.value[this.size++] = value;
        }
    }

    has(value) {
        for (let item of this.value) {
            if (item == value) {
                return true;
            }
        }
        return false;
    }
    valueOf() {
        return this.value;
    }

    sort() {
        this.value.sort();
    }
    toString() {
        return this.value + "";
    }
}
