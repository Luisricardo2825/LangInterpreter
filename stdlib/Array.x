class Collection {
    value = [];
    length = 0;
    constructor(self, ...args) {
        self.value = args;
        self.length = len(args);
    }

    iter(self) {
        return self.value;
    }

    enumerate(self) {
        let chars = [];
        for (let i = 0; i < self.length; i++) {
            chars.push([i, self.value[i]]);
        }
        return chars;
    }

    valueOf(self) {
        return self.value;
    }
}
