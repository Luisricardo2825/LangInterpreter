let input = new Number(Io.readln("Digite um numero:", 0));
let validos = [1, 2, 3, 4, 5];

if (input == 0) {
    Io.println("O numero é zero");
} else if (input in validos) {
    Io.println("O numero é válido");
} else {
    Io.println("O numero não é válido");
}
