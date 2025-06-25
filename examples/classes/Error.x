class Error {
    name = "Error";
    message = "Default error message";
    constructor(self, name, message) {
        if (name != null) {
            self.name = name;
        }
        if (message != null) {
            self.message = message;
        }
    }

    static throw(name, message) {
        return new Error(name, message);
    }
    paint(self) {
        let redName = "\x1b[31m" + self.name + "\x1b[0m";
        return redName + ": " + self.message;
    }
    toString(self) {
        return self.paint();
    }

    valueOf(self) {
        return self.toString();
    }

    getMessage(self) {
        return self.message;
    }

    getName(self) {
        return self.name;
    }

    setName(self, name) {
        self.name = name;
    }

    setMessage(self, message) {
        self.message = message;
    }
}
