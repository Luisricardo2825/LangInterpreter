let object = {
    name: "John",
    age: 30,
};

object.address = {
    street: "123 Main St",
    city: "New York",
};

let jsonString = Json.stringify(object);

Io.println(jsonString);
