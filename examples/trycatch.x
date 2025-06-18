let a = [10, new Error("Teste"), 3];
let b = {
    a: 10,
    b: "Olá,",
    c: {
        d: 10,
        e: 20,
        f: {
            g: 30,
        },
    },
};
a[0] += 10;

b["c"]["f"]["g"] += 30;
b["c"]["f"]["g"] += 30;
b["b"] += " Mundo!";
b.b += " Mundo!";
// b.b += " Mundo!"; ainda não é suportado

try {
    throw new Error("Isso foi esperado");
} catch (e) {
    Io.println(e);
}
Io.println(a[0], a[1].getMessage(), b);
// TestsException.throw("Jogado");
