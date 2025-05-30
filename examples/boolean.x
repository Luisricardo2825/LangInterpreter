let arr = [1, 2, 3, 4];
let obj = { a: 1, b: 2, c: 3 };

if ("a" in obj) {
    Io.println("'a' está no objeto");
} else {
}

if ("j" in obj) {
    Io.println("'a' não está no objeto");
} else {
    Io.println("'j' não está no objeto");
}

if (1 in arr) {
    Io.println("'1' está no array");
} else {
    Io.println("'1' não está no array");
}

if (5 in arr) {
    Io.println("'5' está no array");
} else {
    Io.println("'5' não está no array");
}
