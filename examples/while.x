let max = 100_000;
let count = 0;
while (count < max) {
    count++;
}
Io.println("Finalizado", count);

// Leva cerca de 1,7 minutos para iterar sobre 10 milhoes
// 100 milhoes leva cerca de 12 minutos :O
// 1 bilhão é inviavel, pois deve levar cerca de 2 horas :O