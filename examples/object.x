import { Date } from "./examples/classes/Date.x";

class Todo {
    id;
    title;
    createdAt;
    constructor(id, title) {
        this.id = id;
        this.title = title;
        this.createdAt = new Date();
    }

    constructor(id, title,createdAt) {
        this.id = id;
        this.title = title;
        this.createdAt = createdAt;
    }

    getTitle() {
        return this.title;
    }
    setTitle(title) {
        this.title = title;
    }
    toString() {
        return "Todo: " + id + " - " + title + " - " + createdAt.toString();
    }
}

let todo = new Todo(1, "Read a book", new Date());
