class String extends Collection {
    value = null;
    length = 0;
    constructor(self, value) {
        self.value = value;
        self.length = len(value);
    }

    static factory(nome) {
        return new String(nome);
    }

    charAt(self, index) {
        for (let i = 0; i < self.length; i++) {
            if (i == index) {
                return self.value[i];
            }
        }

        return null;
    }

    concat(self, str) {
        if (!(str instanceof String)) {
            str = new String(str);
        }
        return new String(self.value + str.value);
    }

    @Operator // Marca como operador para a operação "+"(add)
    add(self, str) {
        return self.concat(str);
    }

    replaceCharAt(self, index, char) {
        let result = "";
        for (let i = 0; i < self.length; i++) {
            if (i == index) {
                result += char;
            } else {
                result += self.value[i];
            }
        }
        return new String(result);
    }
    replace(self, searchValue, replaceValue) {
        let result = "";

        let found = false;

        searchValue = new String(searchValue);
        replaceValue = new String(replaceValue);

        for (let i = 0; i < self.length; {}) {
            if (!found && self.value[i] == searchValue.value[0]) {
                let j = 0;
                for (let _ = 0; j < searchValue.length; j++) {
                    if (self.value[i + j] != searchValue.value[j]) {
                        break;
                    }
                }

                if (j == searchValue.length) {
                    if (replaceValue.value != null) {
                        result = result + replaceValue.value;
                    } else {
                        result = replaceValue.value;
                    }
                    i += searchValue.length;
                    found = true;
                    continue;
                }
            }

            if (self.value[i] != null) {
                result = result + self.value[i];
            }

            i = i + 1;
        }

        return new String(result);
    }

    static factory(value) {
        return new String(value);
    }
    indexOf(self, str) {
        // find index of str using a for-loop
        for (let i = 0; i < self.length; i++) {
            if (self.value[i] == str) {
                return i;
            }
        }
        return -1;
    }

    lastIndexOf(self, str) {
        // find last index of str using a for-loop
        for (let i = self.length; i >= 0; i--) {
            if (self.value[i] == str) {
                return i;
            }
        }
        return -1;
    }

    getLength(self) {
        return self.length;
    }

    toString(self) {
        return self.value;
    }

    valueOf(self) {
        return self.value;
    }

    chars(self) {
        let chars = [];
        for (let i = 0; i < self.length; i++) {
            chars.push(self.value[i]);
        }
        return chars;
    }

    iter(self) {
        return self.chars();
    }
    enumerate(self) {
        let chars = [];
        for (let i = 0; i < self.length; i++) {
            chars.push([i, self.value[i]]);
        }
        return chars;
    }

    repeat(self, count) {
        let result = "";
        for (let i = 0; i < count; i++) {
            result += self.value;
        }
        return new String(result);
    }
}
