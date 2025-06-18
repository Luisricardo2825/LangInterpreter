let validos = [1, 2, 3, 4, 5];

for (let i of validos) {
    let idx = validos.indexOf(i);
    if (i == 0) {
        Io.println("O numero é zero", i);
    } else if (idx in validos) {
        Io.println("O numero é válido", i);
    } else {
        Io.println("O numero não é válido", i, idx in validos);
    }
}
