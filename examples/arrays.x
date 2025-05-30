let pessoas = [];
let ids = [1,2,3,4,5,6,7,8,9,10];
for (let idx of ids) {
  pessoas.push({ nome: "Teste " + idx, id: idx });
}

Io.println(pessoas)
