let array = [1, 2, 3, 4];
let count = 0;
for (let i = 0; true; i++) {
    if (i >= 10) {
        Io.println("é maior que", i);
        break;
    }

    Io.println("Valor de i:", i);
}

try {
    throw new Error("TesteExc", "Teste de erro");
} catch (e) {
    Io.println("Ocorreu um error", e);
}
