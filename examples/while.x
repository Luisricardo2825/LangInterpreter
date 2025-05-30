let max = new Number(Io.readln("Digite até onde será contado:", 10));

let stopAt = new Number(Io.readln("Digite um para parar:", 10));
let count = 0;
while (count <= max) {
    Io.println("Count esta em:", count);
    count = count + 1;
    if (count == stopAt) {
        Io.println("Break. Count esta em:", count);
        break;
    }
}
Io.println("Finalizado");
