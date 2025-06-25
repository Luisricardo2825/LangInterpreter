class Str {
    value = null;
    length = 0;
    constructor(self, value) {
        self.value = value;
        self.length = len(value);
    }

    static factory(nome) {
        return new Str(nome);
    }

    charAt(self, index) {
        for (let i = 0; i < self.length; i++) {
            if (i == index) {
                return self.value[i];
            }
        }
    }

    concat(self, str) {
        if (!(str instanceof Str)) {
            str = new Str(str);
        }
        return new Str(self.value + str.value);
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
        return new Str(result);
    }
    replace(self, searchValue, replaceValue) {
        let result = "";

        let found = false;

        searchValue = new Str(searchValue);
        replaceValue = new Str(replaceValue);

        for (let i = 0; i < self.length; {}) {
            if (!found && self.value[i] == searchValue.value[0]) {
                let j = 0;
                for (let ___ = 0; j < searchValue.length; j++) {
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

            result = result + self.value[i];
            i = i + 1;
        }

        return new Str(result);
    }

    static factory(value) {
        return new Str(value);
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
        return self.value + " Esse é o valueOF";
    }

    setValue(self, value) {
        self.value = value;
    }

    clone(self) {
        return new Str(self.value);
    }
}

class Todo {
    id = null;
    title = null;
    userId = null;
    completed = null;

    constructor(self, id, title, userId, completed) {
        self.id = id;
        self.title = title;
        self.userId = userId;
        self.completed = completed;
    }

    toString(self) {
        let completed = "No";

        if (self.completed) {
            completed = "Yes";
        }
        return "Todo(" + self.id + "): " + self.title + " Completed? " + completed;
    }
}
let itens = JSON.parse(Fs.readFile("./todos.json"));
let todos = new Array();
for (let element of itens) {
    let todo = new Todo(element["id"], element["title"], element["userId"], element["completed"]);
    todos.push(todo);
}

function teste() {
    let last = null;
    let count = 0;
    for (let todo of todos) {
        Fs.writeLine("todos.txt", todo);
        // Io.println(todo);
        last = todo;
        count++;
    }
    return [last, count];
}

let resultado = teste();
let item = resultado[0];
let a = JSON.parse(`{
  "user": {
    "id": 12345,
    "name": "Ricardo Alves",
    "email": "ricardo@example.com",
    "is_active": true,
    "roles": ["admin", "editor"],
    "profile": {
      "bio": "Desenvolvedor full stack com experiência em Rust, Java e JS.",
      "social": {
        "github": "https://github.com/ricardoalves",
        "linkedin": "https://linkedin.com/in/ricardoalves"
      },
      "preferences": {
        "theme": "dark",
        "notifications": {
          "email": true,
          "sms": false,
          "push": true
        }
      }
    }
  },
  "posts": [
    {
      "id": 1,
      "title": "Introdução ao Rust",
      "tags": ["rust", "sistemas", "desempenho"],
      "comments": [
        {
          "user": "joao123",
          "message": "Ótimo post!",
          "likes": 12
        },
        {
          "user": "ana_dev",
          "message": "Muito bem explicado.",
          "likes": 8
        }
      ]
    },
    {
      "id": 2,
      "title": "Spring Boot na prática",
      "tags": ["java", "spring"],
      "comments": []
    }
  ],
  "metadata": {
    "generated_at": "2025-06-25T02:30:00Z",
    "server": "api-v2",
    "flags": null
  }
}`);
Io.println("Io.println", item instanceof Todo, JSON.stringify(a), resultado[1]);
