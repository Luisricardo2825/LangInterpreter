let pessoas = [];

for (let idx of range(1, 10)) {
  pessoas.push({ nome: "Teste " + idx, id: idx });
}

let arr = [];
arr[0] = Io;
arr[0].println(pessoas);
